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

import ua.nanit.limbo.protocol.PacketOut;
import ua.nanit.limbo.protocol.registry.State;
import ua.nanit.limbo.protocol.registry.Version;

import java.util.function.Function;

/**
 * One clientbound packet to dump, together with the state that decides its id.
 *
 * <p>The factory takes a version because a few packets carry version-dependent content
 * rather than merely version-dependent framing: known packs names the release, and
 * update tags differs per registry generation.
 */
public final class PacketFixture {

    private final String name;
    private final State state;
    private final Class<? extends PacketOut> packetClass;
    private final Function<Version, PacketOut> factory;

    public PacketFixture(String name,
                         State state,
                         Class<? extends PacketOut> packetClass,
                         Function<Version, PacketOut> factory) {
        this.name = name;
        this.state = state;
        this.packetClass = packetClass;
        this.factory = factory;
    }

    public String getName() {
        return this.name;
    }

    public State getState() {
        return this.state;
    }

    /**
     * The id this packet travels under for the given version, or -1 when the server
     * never sends it to that version.
     */
    public int packetId(Version version) {
        // getRegistry returns null for a state the version predates - Configuration did
        // not exist before 1.20.2. The server silently drops such packets in its encoder;
        // here it simply means the fixture does not apply to this version.
        State.PacketRegistry registry = this.state.clientBound.getRegistry(version);
        return registry == null ? -1 : registry.getPacketId(this.packetClass);
    }

    public PacketOut create(Version version) {
        return this.factory.apply(version);
    }
}
