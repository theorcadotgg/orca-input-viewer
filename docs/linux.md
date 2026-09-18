# Orca Input Viewer on Linux

Linux builds are x86_64 and need Melee NTSC 1.02 running in Slippi or Dolphin
(64-bit). Input mode is the same as on the other platforms: **Dolphin** reads
your inputs out of the emulator's memory, **Standalone** reads them from the USB
adapter.

## Install

- **AppImage** — works on any distro:
  ```sh
  chmod +x Orca*.AppImage && ./Orca*.AppImage
  ```
  If it refuses to start, FUSE 2 is missing: install `libfuse2`
  (Debian/Ubuntu) or `fuse2` (Arch), or run
  `./Orca*.AppImage --appimage-extract-and-run`.
- **.deb** (Debian/Ubuntu) and **.rpm** (Fedora/RHEL) are also published. Only
  the AppImage can update itself from inside the app.

## Dolphin mode: one-time permission

Dolphin mode reads inputs out of Slippi's emulated memory. Linux does not let one
program read another program's memory unless ptrace access is allowed, so the
first time you press **Start Stream** with Slippi running, the viewer asks for
your password once, applies the change, and starts the stream.

- Prefer to do it yourself:
  ```sh
  sudo sysctl -w kernel.yama.ptrace_scope=0
  ```
  To keep it across reboots, put `kernel.yama.ptrace_scope=0` in
  `/etc/sysctl.d/99-orca-input-viewer.conf`.
- **No password prompt appears?** Your desktop has no polkit authentication agent
  and the viewer prints the command above instead. GNOME, KDE and XFCE ship an
  agent; on Hyprland/sway install `hyprpolkitagent` or `polkit-gnome`.
- Declining is not sticky: the next **Start Stream** asks again.

The setting is the one debuggers and capture tools use: it lets programs running
as *you* read the memory of other programs running as *you*.

## USB adapter, and loading device config

Standalone (USB adapter) mode needs no ptrace permission, but the adapter and
Orca's config port are root-owned unless a udev rule says otherwise. One-time
setup, then replug both devices:

```sh
sudo tee /etc/udev/rules.d/51-orca-input-viewer.rules >/dev/null <<'EOF'
SUBSYSTEM=="usb", ATTRS{idVendor}=="057e", ATTRS{idProduct}=="0337", TAG+="uaccess"
SUBSYSTEM=="tty", ATTRS{idVendor}=="2e8a", ATTRS{idProduct}=="000a", TAG+="uaccess"
EOF
sudo udevadm control --reload-rules && sudo udevadm trigger
```

**Load Device Config** needs the same rules, since it talks to Orca over USB.
Approving the Dolphin-mode prompt installs these rules too when they are missing.

## Streaming to OBS

On Wayland the overlay window cannot stay on top — the compositor has no such
hint, so the overlay's **Pin** button does nothing there. Use **Outputs → Start
server** and point OBS at the URL it shows (Add source → Browser). On X11 the
overlay window behaves as it does on Windows and macOS.

## Using it

1. Launch the viewer and leave **Input mode** on **Dolphin**.
2. Press **Start Stream** — it shows "Waiting for Dolphin/Slippi to report inputs".
3. Start Slippi and load Melee; the status chip turns **Live** and the diagram
   follows your controller.

## Troubleshooting

| Symptom | Fix |
| --- | --- |
| The app will not launch | FUSE 2 missing — install `libfuse2`/`fuse2`, or run it with `--appimage-extract-and-run` |
| "Linux blocked access to Dolphin's memory" | Approve the password prompt; if none appears, run the `sysctl` command above |
| Stuck on "Waiting for Dolphin/Slippi" | The emulator is not running yet — start Slippi |
| "Could not find GameCube RAM" | Dolphin is running but no game is loaded — load Melee, then Start Stream again |
| Adapter inputs do not show in Standalone mode | Apply the udev rules, then replug the adapter |
| Overlay window will not stay on top | Wayland — use the OBS browser source instead |
| Overlay is black or garbled with an NVIDIA card | Handled automatically in current builds; older builds need `WEBKIT_DISABLE_DMABUF_RENDERER=1` |
