#!/usr/bin/env bash
#
# Linux setup for Orca Input Viewer.
#
# Two things need fixing on a stock install:
#
#   1. USB access. The GameCube adapter (standalone/USB input mode) and Orca's
#      RP2040 config interface are root-owned by default, so the viewer cannot
#      open them as your user.
#   2. Dolphin memory access. Dolphin mode reads the emulator's emulated RAM,
#      which the kernel gates behind ptrace permission (kernel.yama.ptrace_scope).
#
# Run this script once, then replug the devices.

set -euo pipefail

RULES_FILE=/etc/udev/rules.d/51-orca-input-viewer.rules

if [ "${EUID}" -eq 0 ]; then
  echo "Run this as your normal user; it will call sudo where needed." >&2
  exit 1
fi

echo "== Installing udev rules ($RULES_FILE) =="
sudo tee "$RULES_FILE" >/dev/null <<'RULES'
# Orca Input Viewer: GameCube adapter, used for standalone (USB) input mode.
SUBSYSTEM=="usb", ATTRS{idVendor}=="057e", ATTRS{idProduct}=="0337", TAG+="uaccess"

# Orca Input Viewer: RP2040 config interface, a CDC serial device.
SUBSYSTEM=="tty", ATTRS{idVendor}=="2e8a", ATTRS{idProduct}=="000a", TAG+="uaccess"
RULES

sudo udevadm control --reload-rules
sudo udevadm trigger
echo "Reloaded udev. Replug the adapter and the Orca board."

echo
echo "== Dolphin memory access =="
ptrace_scope=$(cat /proc/sys/kernel/yama/ptrace_scope 2>/dev/null || echo 0)
if [ "$ptrace_scope" = "0" ]; then
  echo "kernel.yama.ptrace_scope is 0 - Dolphin mode can read the emulator's RAM."
else
  cat <<'EOF'
kernel.yama.ptrace_scope is 1, so reading Dolphin's memory is denied. Pick one:

  AppImage  - allow ptrace system-wide (persist with /etc/sysctl.d/99-orca.conf):
                sudo sysctl -w kernel.yama.ptrace_scope=0

  .deb/.rpm - grant just this binary the capability, which is narrower:
                sudo setcap cap_sys_ptrace=eip "$(command -v orca-input-viewer)"

Without this the viewer still works in standalone (USB adapter) mode.
EOF
fi
