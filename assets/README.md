# assets

Images referenced by the root `README.md`. Drop the real files here under exactly these
names and the README picks them up — nothing else needs editing.

Until a file exists GitHub renders a broken-image icon in its place. Every reference is
sized and centred, so a missing file leaves a gap rather than breaking the layout.

| File                     | Used for                                                                        | Recommended size                           | Notes                                                                                                                           |
|--------------------------|---------------------------------------------------------------------------------|--------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------|
| `logo.png`               | Hero logo at the top of the README                                              | 512×512 px, square, transparent background | Rendered at 160 px wide. Keep it legible when small, and readable on both light and dark GitHub themes.                         |
| `preview-serverlist.png` | Multiplayer server list entry showing the MOTD, player count and version string | 1280×200 px                                | Crop to just the server entry. Uses the shipped `settings.yml` gradients, so it doubles as documentation of `ping.description`. |
| `preview-ingame.png`     | A player parked in the limbo world                                              | 1280×720 px, 16:9                          | The point is the boss bar / title / tab list configured in `settings.yml`, not the scenery.                                     |
| `preview-console.png`    | Terminal running the server, with `help`, `conn` and `mem` typed                | 1280×720 px                                | A dark terminal theme reads best. Trim anything above the startup banner.                                                       |

Guidelines:

- PNG, not JPEG — screenshots of UI and text compress badly as JPEG.
- Keep each file under ~500 KB so the README loads quickly; run them through `oxipng` or
  `pngquant` if needed.
- Capture at 2× and downscale, so text stays sharp on high-DPI displays.
- No personal information in frame: real usernames, IP addresses, or server addresses in
  the multiplayer list.
