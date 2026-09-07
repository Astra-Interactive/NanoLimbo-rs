/*
 * Copyright (C) 2020 Nan1t
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

package ua.nanit.limbo.tools;

import com.google.gson.GsonBuilder;
import io.netty.buffer.Unpooled;
import net.kyori.adventure.text.Component;
import ua.nanit.limbo.LimboConstants;
import ua.nanit.limbo.protocol.ByteMessage;
import ua.nanit.limbo.protocol.PacketOut;
import ua.nanit.limbo.protocol.packets.configuration.PacketFinishConfiguration;
import ua.nanit.limbo.protocol.packets.configuration.PacketKnownPacks;
import ua.nanit.limbo.protocol.packets.configuration.PacketRegistryData;
import ua.nanit.limbo.protocol.packets.login.PacketLoginDisconnect;
import ua.nanit.limbo.protocol.packets.login.PacketLoginSuccess;
import ua.nanit.limbo.protocol.packets.play.*;
import ua.nanit.limbo.protocol.registry.State;
import ua.nanit.limbo.protocol.registry.Version;
import ua.nanit.limbo.server.LimboServer;
import ua.nanit.limbo.server.data.BossBar;
import ua.nanit.limbo.util.ComponentUtils;
import ua.nanit.limbo.util.UUIDUtils;
import ua.nanit.limbo.world.DimensionRegistry;
import ua.nanit.limbo.world.DimensionType;
import ua.nanit.limbo.world.VersionedDimension;

import java.util.Collections;
import java.util.UUID;

import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.TreeSet;

/**
 * Dumps the byte-level output of this implementation so the Rust port can be checked
 * against it. This is the level 1 oracle: byte-for-byte reference output.
 *
 * <p>Not part of the server. Run it explicitly:
 * <pre>java -cp build/libs/NanoLimbo.jar ua.nanit.limbo.tools.FixtureDumper rust/fixtures</pre>
 *
 * <p>Everything dumped here must be deterministic. Anything drawn from a random source
 * or a clock is passed in as a fixed value rather than generated.
 */
public final class FixtureDumper {

    /**
     * Text as it can appear in settings.yml. Covers every MiniMessage construct the
     * default configuration relies on, both legacy colour code styles, raw JSON input,
     * and the edge cases a hand-written parser tends to get wrong.
     */
    private static final List<String> TEXT_INPUTS = List.of(
            "",
            "plain text",
            "NanoLimbo",
            "&aGreen &lBold&r plain",
            "&x&f&f&0&0&0&0hex legacy",
            "§aSection green",
            "<red>red text",
            "<#ff8800>hex colour",
            "<gradient:blue:white>NanoLimbo",
            "<gradient:red:green:blue>three stops",
            "<gradient:blue:white>x",
            "<rainbow>rainbow text",
            "<bold>bold</bold> normal",
            "<b>b</b><i>i</i><u>u</u><st>st</st><obf>obf</obf>",
            "<red>outer <blue>inner</blue> outer",
            "<red>unclosed",
            "\\<not a tag>",
            "<click:open_url:'https://example.com'>link</click>",
            "<click:run_command:'/help'>run</click>",
            "<hover:show_text:'tooltip'>hover me</hover>",
            "<key:key.jump>",
            "<lang:menu.singleplayer>",
            "line one<newline>line two",
            "<white>unicode: ä€𝄞",
            "{\"text\":\"raw json\",\"color\":\"red\"}",
            "{\"text\":\"bold json\",\"bold\":true,\"color\":\"#00ff00\"}",
            "<gradient:blue:white>NanoLimbo",
            "<white>Welcome to the <gradient:blue:white>NanoLimbo<white>!",
            "<white>Welcome!",
            "<white><b>Welcome!",
            "unstyled text for the compact form",
            "<red><hover:show_text:'<blue>styled tip'>styled hover</hover>",
            "<click:suggest_command:'/login '>suggest</click>",
            "<click:copy_to_clipboard:'copied'>copy</click>",
            "<red>a<blue>b</blue><green>c</green>"
    );

    private static final List<String> USERNAMES = List.of(
            "NanoLimbo",
            "Notch",
            "a",
            "0123456789abcdef",
            "äöü",
            "𝄞"
    );

    /**
     * One version per JSON serializer profile. Two of these never put JSON on the wire —
     * from 1.20.3 components travel as NBT — but the login disconnect and status response
     * packets serialize to a JSON string on every version, so all four profiles are live.
     */
    private static final List<Version> JSON_PROFILE_REPRESENTATIVES = List.of(
            Version.V1_7_2, Version.V1_16, Version.V1_20_3, Version.V1_21_5);

    private static final UUID PLAYER_UUID = UUIDUtils.getOfflineModeUuid("NanoLimbo");
    private static final UUID SESSION_UUID =
            UUID.fromString("00000000-0000-4000-8000-000000000001");
    private static final UUID BOSS_BAR_UUID =
            UUID.fromString("00000000-0000-4000-8000-000000000002");
    private static final UUID CHAT_SENDER_UUID =
            UUID.fromString("00000000-0000-4000-8000-000000000003");
    private static final int ENTITY_ID = 1337;
    private static final int TELEPORT_ID = 7654321;

    private FixtureDumper() {
    }

    private static String toHex(byte[] bytes) {
        StringBuilder hex = new StringBuilder(bytes.length * 2);
        for (byte value : bytes) {
            hex.append(String.format("%02x", value));
        }
        return hex.toString();
    }

    private static byte[] encodeComponent(Component component, Version version) {
        ByteMessage message = new ByteMessage(Unpooled.buffer());
        try {
            message.writeComponent(component, version);
            return message.toByteArray();
        } finally {
            message.release();
        }
    }

    /**
     * Groups versions by the bytes they produce. Only a handful of distinct encodings
     * exist across the 51 versions, so grouping keeps the fixture small and makes the
     * version boundaries obvious to a reader.
     */
    private static List<Map<String, Object>> encodeForAllVersions(Component component) {
        Map<String, List<Integer>> protocolsByHex = new LinkedHashMap<>();

        for (Version version : Version.values()) {
            if (!version.isSupported()) {
                continue;
            }
            String hex = toHex(encodeComponent(component, version));
            protocolsByHex.computeIfAbsent(hex, key -> new ArrayList<>())
                    .add(version.getProtocolNumber());
        }

        List<Map<String, Object>> encodings = new ArrayList<>();
        for (Map.Entry<String, List<Integer>> entry : protocolsByHex.entrySet()) {
            Map<String, Object> encoding = new LinkedHashMap<>();
            encoding.put("protocols", entry.getValue());
            encoding.put("hex", entry.getKey());
            encodings.add(encoding);
        }
        return encodings;
    }

    private static void dumpComponents(Path root) throws Exception {
        List<Map<String, Object>> entries = new ArrayList<>();

        for (String input : TEXT_INPUTS) {
            Component component = ComponentUtils.parse(input);

            Map<String, Object> profiles = new LinkedHashMap<>();
            for (Version representative : JSON_PROFILE_REPRESENTATIVES) {
                profiles.put(String.valueOf(representative.getProtocolNumber()),
                        ComponentUtils.getJsonChatSerializer(representative).serialize(component));
            }

            Map<String, Object> entry = new LinkedHashMap<>();
            entry.put("input", input);
            entry.put("legacy", ComponentUtils.toLegacyString(component));
            entry.put("plain", ComponentUtils.toPlainString(component));
            entry.put("json", profiles);
            entry.put("encodings", encodeForAllVersions(component));
            entries.add(entry);
        }

        Map<String, Object> document = new LinkedHashMap<>();
        document.put("description",
                "Output of ByteMessage.writeComponent for each settings.yml text input, "
                        + "grouped by the versions that share an encoding.");
        document.put("entries", entries);

        write(root.resolve("text/components.json"), document);
    }

    private static void dumpOfflineUuids(Path root) throws Exception {
        List<Map<String, Object>> entries = new ArrayList<>();

        for (String username : USERNAMES) {
            Map<String, Object> entry = new LinkedHashMap<>();
            entry.put("username", username);
            entry.put("uuid", UUIDUtils.getOfflineModeUuid(username).toString());
            entries.add(entry);
        }

        Map<String, Object> document = new LinkedHashMap<>();
        document.put("description",
                "UUID.nameUUIDFromBytes(\"OfflinePlayer:\" + name), the offline-mode identity.");
        document.put("entries", entries);

        write(root.resolve("uuid/offline.json"), document);
    }

    private static void write(Path path, Object document) throws Exception {
        Files.createDirectories(path.getParent());
        String json = new GsonBuilder().setPrettyPrinting().disableHtmlEscaping()
                .create().toJson(document);
        Files.write(path, (json + "\n").getBytes(StandardCharsets.UTF_8));
        System.out.printf("wrote %s (%d bytes)%n", path, Files.size(path));
    }

    private static PacketOut joinGame(VersionedDimension dimension) {
        PacketLogin packet = new PacketLogin();
        packet.setEntityId(ENTITY_ID);
        packet.setEnableRespawnScreen(true);
        packet.setFlat(false);
        packet.setGameMode(3);
        packet.setSecureProfile(false);
        packet.setHardcore(false);
        packet.setMaxPlayers(100);
        packet.setPreviousGameMode(-1);
        packet.setReducedDebugInfo(true);
        packet.setDebug(false);
        packet.setViewDistance(0);
        packet.setSeed(0);
        packet.setDimension(dimension);
        return packet;
    }

    private static PacketOut bossBar() {
        BossBar bar = new BossBar();
        bar.setText(ComponentUtils.parse("<white>Welcome to the <gradient:blue:white>NanoLimbo<white>!"));
        bar.setHealth(1.0F);
        bar.setColor(BossBar.Color.BLUE);
        bar.setDivision(BossBar.Division.SOLID);

        PacketBossBar packet = new PacketBossBar();
        packet.setBossBar(bar);
        packet.setUuid(BOSS_BAR_UUID);
        return packet;
    }

    private static List<PacketFixture> packetFixtures(DimensionRegistry registry) {
        VersionedDimension dimension = DimensionType.THE_END.createVersionedDimension(registry);
        List<PacketFixture> fixtures = new ArrayList<>();

        fixtures.add(new PacketFixture("login_success", State.LOGIN, PacketLoginSuccess.class, version -> {
            PacketLoginSuccess packet = new PacketLoginSuccess();
            packet.setUsername("NanoLimbo");
            packet.setUuid(PLAYER_UUID);
            packet.setSessionId(SESSION_UUID);
            return packet;
        }));
        fixtures.add(new PacketFixture("login_disconnect", State.LOGIN, PacketLoginDisconnect.class, version -> {
            PacketLoginDisconnect packet = new PacketLoginDisconnect();
            packet.setReason(ComponentUtils.parse("<red>Unsupported client version"));
            return packet;
        }));
        fixtures.add(new PacketFixture("join_game", State.PLAY, PacketLogin.class,
                version -> joinGame(dimension)));
        fixtures.add(new PacketFixture("player_abilities", State.PLAY, PacketPlayerAbilities.class, version -> {
            PacketPlayerAbilities packet = new PacketPlayerAbilities();
            packet.setFlyingSpeed(0.0F);
            packet.setFlying(true);
            packet.setFieldOfView(0.1F);
            return packet;
        }));
        fixtures.add(new PacketFixture("position_and_look_legacy", State.PLAY, PacketPlayerPositionAndLook.class,
                version -> new PacketPlayerPositionAndLook(0, 64, 0, 0, 0, TELEPORT_ID)));
        fixtures.add(new PacketFixture("position_and_look", State.PLAY, PacketPlayerPositionAndLook.class,
                version -> new PacketPlayerPositionAndLook(0, 400, 0, 0, 0, TELEPORT_ID)));
        fixtures.add(new PacketFixture("spawn_position", State.PLAY, PacketSpawnPosition.class,
                version -> new PacketSpawnPosition(dimension.getKey(), 0, 400, 0, 0, 0)));
        fixtures.add(new PacketFixture("player_info", State.PLAY, PacketPlayerInfo.class, version -> {
            PacketPlayerInfo packet = new PacketPlayerInfo();
            packet.setUsername("NanoLimbo");
            packet.setGameMode(3);
            packet.setUuid(PLAYER_UUID);
            return packet;
        }));
        fixtures.add(new PacketFixture("declare_commands", State.PLAY, PacketDeclareCommands.class, version -> {
            PacketDeclareCommands packet = new PacketDeclareCommands();
            packet.setCommands(Collections.emptyList());
            return packet;
        }));
        fixtures.add(new PacketFixture("plugin_message_play", State.PLAY, PacketPluginMessage.class,
                version -> brandPluginMessage()));
        fixtures.add(new PacketFixture("plugin_message_configuration", State.CONFIGURATION,
                PacketPluginMessage.class, version -> brandPluginMessage()));
        fixtures.add(new PacketFixture("chat_message", State.PLAY, PacketChatMessage.class, version -> {
            PacketChatMessage packet = new PacketChatMessage();
            packet.setMessage(ComponentUtils.parse("<white>Welcome to the <gradient:blue:white>NanoLimbo<white>!"));
            packet.setPosition(PacketChatMessage.PositionLegacy.SYSTEM_MESSAGE);
            packet.setSender(CHAT_SENDER_UUID);
            return packet;
        }));
        fixtures.add(new PacketFixture("boss_bar", State.PLAY, PacketBossBar.class, version -> bossBar()));
        fixtures.add(new PacketFixture("player_list_header", State.PLAY, PacketPlayerListHeader.class, version -> {
            PacketPlayerListHeader packet = new PacketPlayerListHeader();
            packet.setHeader(ComponentUtils.parse("<white>Welcome!"));
            packet.setFooter(ComponentUtils.parse("<gradient:blue:white>NanoLimbo"));
            return packet;
        }));
        fixtures.add(new PacketFixture("title_set_title", State.PLAY, PacketTitleSetTitle.class, version -> {
            PacketTitleSetTitle packet = new PacketTitleSetTitle();
            packet.setTitle(ComponentUtils.parse("<white><b>Welcome!"));
            return packet;
        }));
        fixtures.add(new PacketFixture("title_set_subtitle", State.PLAY, PacketTitleSetSubTitle.class, version -> {
            PacketTitleSetSubTitle packet = new PacketTitleSetSubTitle();
            packet.setSubtitle(ComponentUtils.parse("<gradient:blue:white>NanoLimbo"));
            return packet;
        }));
        fixtures.add(new PacketFixture("title_times", State.PLAY, PacketTitleTimes.class,
                version -> new PacketTitleTimes(10, 100, 10)));
        fixtures.add(new PacketFixture("disconnect_play", State.PLAY, PacketDisconnect.class, version -> {
            PacketDisconnect packet = new PacketDisconnect();
            packet.setReason(ComponentUtils.parse("<red>Too many players connected"));
            return packet;
        }));
        fixtures.add(new PacketFixture("game_event", State.PLAY, PacketGameEvent.class,
                version -> new PacketGameEvent((byte) 13, 0)));
        fixtures.add(new PacketFixture("chunk_with_light", State.PLAY, PacketChunkWithLight.class, version -> {
            PacketChunkWithLight packet = new PacketChunkWithLight();
            packet.setX(0);
            packet.setZ(0);
            packet.setDimension(dimension);
            return packet;
        }));
        fixtures.add(new PacketFixture("finish_configuration", State.CONFIGURATION,
                PacketFinishConfiguration.class, version -> new PacketFinishConfiguration()));
        fixtures.add(new PacketFixture("known_packs", State.CONFIGURATION, PacketKnownPacks.class, version -> {
            PacketKnownPacks packet = new PacketKnownPacks();
            packet.setKnownPacks(List.of(new PacketKnownPacks.KnownPack(
                    "minecraft", "core", version.getDisplayName())));
            return packet;
        }));
        fixtures.add(new PacketFixture("registry_data_legacy", State.CONFIGURATION, PacketRegistryData.class,
                version -> {
                    PacketRegistryData packet = new PacketRegistryData();
                    packet.setMetadataWriter((message, ver) ->
                            message.writeCompoundTag(registry.getCodec_1_20(), ver));
                    return packet;
                }));

        return fixtures;
    }

    private static PacketOut brandPluginMessage() {
        PacketPluginMessage packet = new PacketPluginMessage();
        packet.setChannel(LimboConstants.BRAND_CHANNEL);
        ByteMessage payload = new ByteMessage(Unpooled.buffer());
        try {
            payload.writeString(ComponentUtils.toLegacyString(
                    ComponentUtils.parse("<gradient:blue:white>NanoLimbo")));
            packet.setData(payload.toByteArray());
        } finally {
            payload.release();
        }
        return packet;
    }

    private static void dumpPackets(Path root) throws Exception {
        DimensionRegistry registry = new DimensionRegistry(new LimboServer());
        registry.load();
        dumpUpdateTags(root, registry);

        List<Map<String, Object>> entries = new ArrayList<>();

        for (PacketFixture fixture : packetFixtures(registry)) {
            Map<String, List<Integer>> protocolsByHex = new LinkedHashMap<>();
            Map<Integer, List<Integer>> protocolsById = new LinkedHashMap<>();

            for (Version version : Version.values()) {
                if (!version.isSupported()) {
                    continue;
                }
                int packetId = fixture.packetId(version);
                if (packetId < 0) {
                    continue;
                }

                ByteMessage message = new ByteMessage(Unpooled.buffer());
                String hex;
                try {
                    fixture.create(version).encode(message, version);
                    hex = toHex(message.toByteArray());
                } finally {
                    message.release();
                }

                protocolsByHex.computeIfAbsent(hex, key -> new ArrayList<>())
                        .add(version.getProtocolNumber());
                protocolsById.computeIfAbsent(packetId, key -> new ArrayList<>())
                        .add(version.getProtocolNumber());
            }

            List<Map<String, Object>> encodings = new ArrayList<>();
            for (Map.Entry<String, List<Integer>> entry : protocolsByHex.entrySet()) {
                Map<String, Object> encoding = new LinkedHashMap<>();
                encoding.put("protocols", entry.getValue());
                encoding.put("hex", entry.getKey());
                encodings.add(encoding);
            }

            List<Map<String, Object>> ids = new ArrayList<>();
            for (Map.Entry<Integer, List<Integer>> entry : protocolsById.entrySet()) {
                Map<String, Object> id = new LinkedHashMap<>();
                id.put("protocols", entry.getValue());
                id.put("packetId", entry.getKey());
                ids.add(id);
            }

            Map<String, Object> entry = new LinkedHashMap<>();
            entry.put("name", fixture.getName());
            entry.put("state", fixture.getState().name());
            entry.put("ids", ids);
            entry.put("encodings", encodings);
            entries.add(entry);
        }

        Map<String, Object> document = new LinkedHashMap<>();
        document.put("description",
                "Clientbound packet payloads (without the id prefix) for every version the "
                        + "server sends them to, grouped by shared encoding. Also records the "
                        + "packet id per version as a third check on the id tables. "
                        + "Update tags is deliberately absent - see update_tags.json.");
        document.put("entries", entries);

        write(root.resolve("packets/clientbound.json"), document);
    }

    /**
     * Update tags is dumped as a digest rather than as bytes.
     *
     * <p>{@code DimensionRegistry.parseUpdateTags} collects into a {@code HashMap}, so the
     * order this packet serializes in is an artifact of Java's hashing, not something the
     * protocol specifies. Comparing bytes against it would demand the port reproduce that
     * hashing, which is neither possible nor desirable. What the port must agree on is the
     * content, so that is what this records: per-registry counts to localize a mismatch,
     * and a digest over a canonical ordering to detect one.
     */
    private static void dumpUpdateTags(Path root, DimensionRegistry registry) throws Exception {
        List<Map<String, Object>> entries = new ArrayList<>();

        for (Version version : Version.values()) {
            if (!version.isSupported() || version.less(Version.V1_20_5)) {
                continue;
            }

            Map<String, Map<String, List<Integer>>> tags = registry.createUpdateTags(version);
            StringBuilder canonical = new StringBuilder();
            List<Map<String, Object>> registries = new ArrayList<>();
            int totalIds = 0;

            for (String registryName : new TreeSet<>(tags.keySet())) {
                Map<String, List<Integer>> perRegistry = tags.get(registryName);
                canonical.append(registryName).append('\u0000');
                int registryIds = 0;

                for (String tagName : new TreeSet<>(perRegistry.keySet())) {
                    List<Integer> ids = perRegistry.get(tagName);
                    canonical.append(tagName).append('=').append(ids).append('\u0000');
                    registryIds += ids.size();
                }

                Map<String, Object> summary = new LinkedHashMap<>();
                summary.put("registry", registryName);
                summary.put("tags", perRegistry.size());
                summary.put("ids", registryIds);
                registries.add(summary);
                totalIds += registryIds;
            }

            Map<String, Object> entry = new LinkedHashMap<>();
            entry.put("protocol", version.getProtocolNumber());
            entry.put("registryCount", tags.size());
            entry.put("totalIds", totalIds);
            entry.put("digest", sha256(canonical.toString()));
            entry.put("registries", registries);
            entries.add(entry);
        }

        Map<String, Object> document = new LinkedHashMap<>();
        document.put("description",
                "Update tags content, as per-registry counts plus a SHA-256 over registries "
                        + "and tags sorted by name. Java emits this packet in HashMap order, so "
                        + "its bytes are not comparable - only its content is.");
        document.put("digestInput",
                "for each registry sorted by name: name NUL, then for each tag sorted by name: "
                        + "name '=' List.toString(ids) NUL");
        document.put("entries", entries);

        write(root.resolve("packets/update_tags.json"), document);
    }

    private static String sha256(String value) throws Exception {
        byte[] digest = java.security.MessageDigest.getInstance("SHA-256")
                .digest(value.getBytes(StandardCharsets.UTF_8));
        return toHex(digest);
    }

    public static void main(String[] args) throws Exception {
        Path root = Paths.get(args.length > 0 ? args[0] : "rust/fixtures");
        Files.createDirectories(root);

        dumpComponents(root);
        dumpOfflineUuids(root);
        dumpPackets(root);
    }
}
