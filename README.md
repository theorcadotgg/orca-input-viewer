# Orca Input Viewer

A desktop app that shows what your GameCube controller is doing, live. Every button
press and stick movement is drawn on the Orca controller layout as you play, either in
a small window you can park next to your setup or as a transparent overlay in OBS. It
works with a stock GameCube controller and adapter just as well as with Orca itself.

Your inputs are read one of two ways, chosen in the app:

- **Dolphin** — reads them out of Slippi or Dolphin's own memory, so it keeps working
  while the emulator is the one holding the adapter. This is the mode for emulator and
  online play. Works with Melee (NTSC 1.02).
- **Standalone** — reads a GameCube USB adapter directly, with no emulator in the
  picture. This is the mode for console play, or any setup where the adapter is free.

Installers for Windows, macOS (Apple Silicon) and Linux are published on the
[releases page](https://github.com/theorcadotgg/orca-obs-viewer/releases).

## What you need

- Windows 10/11 (x64), macOS on Apple Silicon, or Linux (x86_64).
- A controller: Orca itself, or any GameCube controller in a GameCube USB adapter (the
  Wii U / Switch-era one). Standalone mode needs that adapter plugged into the PC, with
  its own one-time permission change on Linux — see
  [Platform permissions](#platform-permissions). Dolphin mode needs nothing extra: it
  reads the emulator.
- For Dolphin mode: Slippi or Dolphin running **Melee (NTSC 1.02)**. The memory
  addresses the viewer reads are Melee's.
- For streaming: OBS Studio, or another app that can show a browser source.

## Install

**Windows** — run the `.msi` or the installer `.exe`. Installers are currently unsigned,
so Windows may show a SmartScreen warning: choose *More info → Run anyway*.

**macOS** — open the `.dmg` and drag the app to *Applications*. If macOS refuses the
first launch, right-click the app and choose *Open*.

**Linux** — download the AppImage and run it:

```sh
chmod +x Orca*.AppImage && ./Orca*.AppImage
```

If it will not start, FUSE 2 is missing (`libfuse2` on Debian/Ubuntu, `fuse2` on Arch):
either install that, or run it with `--appimage-extract-and-run`. `.deb` and `.rpm`
packages are published too, but only the AppImage can update itself from inside the app.

The app checks for updates when it launches and offers an **Update & Restart** button;
**Check updates** in the footer does the same on demand.

## Using it

1. Plug the adapter (or Orca) into USB, with a controller connected to port 1.
2. Launch the app. The sidebar has three panels:
   - **Connection** — which port and profile to show, how to read the inputs, and the
     **Start Stream** button.
   - **Device config** — pulls your Orca's own button mappings out of the device.
   - **Outputs** — the on-screen overlay window, and the server OBS connects to.
3. Pick an input mode and press **Start Stream**. The status chip goes
   *Offline → Waiting → Live* as inputs start arriving; the footer shows the frame rate
   of the live feed.

### Port

Which controller port is displayed. In Dolphin mode during Slippi online play the
viewer finds the local player's port itself and locks the dropdown onto it — no need to
follow "whose stock icon is which" in the menus.

### Profile

Orca stores eight profiles. The diagram shows the button mappings of whichever profile
is selected. Before a device config has been loaded it uses the built-in defaults.

### Overlay window

**Outputs → Show** opens a small borderless window with the same diagram.

- Drag anywhere on its title bar to move it.
- **BG** turns the window into a green screen, for capture methods that want a colour to
  key out.
- **Pin** keeps the window on top of other windows (Windows, macOS and X11; Wayland
  gives no always-on-top hint, so use the browser source there).
- **X** hides it; **Hide** in the sidebar does the same.

### OBS browser source

**Outputs → Start server** starts a small web server inside the app and shows a URL.
In OBS, add a **Browser** source, paste the URL, set the size you want, and leave
"Shutdown source when not visible" off. The page draws the diagram on a black
background, so add a **Color Key** (or **Luma Key**) filter with black as the key colour
to lay it over your game capture.

The server listens on `127.0.0.1` only (ports 4725 for the page, 4726 for the live data)
— nothing is exposed to the network. If those ports are busy the app picks free ones and
shows you the URL it actually used.

### Device config

**Device config → Load Device Config** copies the mappings saved on your Orca so the
diagram matches what the device actually does. Put Orca in config mode first; the app
waits for the config port, reads the settings, restarts the device into normal mode, and
remembers the config for next launch (the chip reads *Device mappings* instead of
*Default mappings*). Loading a config needs the same USB permission as Standalone mode —
see below.

## Platform permissions

Two platforms gate the data the viewer wants behind a permission you have to grant once.

### macOS — Dolphin mode

macOS only lets a process read another process's memory if it is entitled to, and
notarized Dolphin/Slippi builds are not. Tick that box with **Enable macOS Access** in
the app (or run `scripts/macos-enable-dolphin-access.sh`): it re-signs the Dolphin/Slippi
install it finds, keeping the entitlements those builds need to run. Restart Dolphin
afterwards.

The first use asks for **App Management** permission (System Settings → Privacy &
Security). Slippi Launcher replaces its Dolphin when it updates, so run this once more
after a launcher update. Standalone mode needs none of this.

### Linux — Dolphin mode and the adapter

Two separate root-owned things have to be loosened:

- **`kernel.yama.ptrace_scope`**, which stops one program from reading another's memory
  (Dolphin mode). The app asks for this itself: on the first **Start Stream** that needs
  it, one password prompt authorises the change and the stream carries on. Declining is
  not sticky.
- **udev rules**, without which the adapter and Orca's config port are root-owned (both
  Standalone mode and device config). Install them with
  `scripts/linux-enable-device-access.sh`, or by hand — [docs/linux.md](docs/linux.md)
  has the exact lines, the Wayland and NVIDIA caveats, and a troubleshooting table.

## Troubleshooting

| Symptom | Likely cause |
| --- | --- |
| Stuck on "Waiting for Dolphin/Slippi to report inputs" | The emulator is not running yet — start Slippi. |
| "Could not find GameCube RAM" | Load Melee NTSC 1.02 and wait until it reaches the menu, then Start Stream again. The viewer only uses a verified RAM mapping. |
| "Dolphin found but couldn't connect" | The memory permission is missing — see [Platform permissions](#platform-permissions). |
| Adapter inputs never appear in Standalone mode | Adapter not seen: replug it, and on Linux install the udev rules. |
| Overlay will not stay on top | Wayland has no always-on-top hint — use the OBS browser source instead. |
| Diagram does not match the buttons you press | Load the device config so the viewer knows Orca's mappings, or pick the right profile. |
| Port dropdown is greyed out | Expected during Slippi online play: the viewer is following the local player's port by itself. |

## Building from source

The app is a [Tauri](https://v2.tauri.app) 2 application: a Rust backend in `src-tauri/`
and plain HTML/CSS/JS in `ui/` that Tauri serves directly. There is no JavaScript build
step and no Node dependency.

Prerequisites are Rust (stable) with [Tauri's system dependencies](https://v2.tauri.app/start/prerequisites/)
for your platform, plus the Tauri CLI:

```sh
cargo install tauri-cli --version '^2' --locked
```

Then, from the repository root:

```sh
cd src-tauri
cargo tauri dev          # run from source
cargo tauri build        # build an installer for this platform
```

Per platform:

- **Windows** — Rust with the MSVC toolchain and Visual Studio's C++ build tools (libusb
  is compiled from source). `cargo tauri build` produces `.msi` and `.exe` installers
  under `src-tauri/target/release/bundle/`.
- **macOS** — Rust plus the Xcode command line tools. `cargo tauri build --bundles app`
  builds the `.app` (leave `--bundles` off for a `.dmg` too). Local builds are ad-hoc
  signed with `src-tauri/entitlements.plist`; the debug entitlement it grants is what
  lets the viewer read the emulator's memory, so keep it. Release builds are Apple
  Silicon only.
- **Linux** — the WebKitGTK development packages (`webkit2gtk-4.1`, plus `librsvg`,
  `pkgconf`, `patchelf` and `xdg-utils` for the AppImage; `libudev` and `libusb` for the
  build). Build the AppImage on the oldest distribution you intend to support — the
  bundle links against that system's WebKitGTK. x86_64 only.

Tests live in `src-tauri`:

```sh
cd src-tauri && cargo test
```

On Linux this includes an integration test that maps a stand-in for Dolphin's memory and
reads controller state out of it, so run it on Linux when touching `dolphin.rs`.

Releases, code signing, notarization and the auto-updater are documented in
[RELEASING.md](RELEASING.md).

## Credits

- **[M'Overlay](https://github.com/bkacjios/m-overlay)** — a Dolphin controller overlay,
  and the inspiration for this one. The overlay window, chroma-key background and OBS
  browser-source idea all come from it.
- **[Slippi](https://github.com/project-slippi)** — the emulator build the viewer reads,
  and the source of the addresses used to work out the local player's port during online
  play (`slippi-ssbm-asm`).
- **[Dolphin](https://github.com/dolphin-emu/dolphin)** — the emulator itself. The
  Melee offsets the reader uses are the community RAM map, the same ones
  [libmelee](https://github.com/altf4/libmelee) carries.
- **[GP2040-CE](https://github.com/OpenStickCommunity/GP2040-CE)** — the open-source
  gamepad firmware Orca's own firmware is built on, and the source of the profile and
  button-mapping model the device-config screen speaks.
- **Orca's own tools** — the Orca web configurator and its shared input library define
  the settings format and mapping tables this viewer decodes, and the controller drawing
  is Orca's own board outline.

## License

MIT — see [LICENSE](LICENSE). The third-party components that end up in a build, and the
conditions their licenses attach, are listed in
[THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md); both files are bundled inside the app so
an installed copy carries its own notices.
