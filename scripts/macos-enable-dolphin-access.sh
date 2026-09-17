#!/usr/bin/env bash
# macOS: grant the Orca Input Viewer access to Dolphin's emulated RAM.
#
# macOS only hands out task ports to entitled processes, so two things must be true:
#   * the viewer carries com.apple.security.cs.debugger  (set in src-tauri/entitlements.plist)
#   * Dolphin/Slippi carries com.apple.security.get-task-allow
# Notarized Dolphin/Slippi builds ship without the second one, so this re-signs the
# local copy with it - keeping the entitlements the build already relies on
# (com.apple.security.cs.allow-jit is what makes Dolphin run on Apple Silicon).
# Slippi Launcher replaces Dolphin on update, so re-run this after an update.
#
# usage: macos-enable-dolphin-access.sh [/path/to/Dolphin.app]
set -euo pipefail

running_dolphin_app() {
    local pattern exe
    for pattern in Slippi_Dolphin dolphin-emu Dolphin; do
        exe=$(pgrep -f "/$pattern" 2>/dev/null | head -1 || true)
        [ -n "$exe" ] || continue
        exe=$(ps -o comm= -p "$exe" 2>/dev/null || true)
        case "$exe" in
            *.app/Contents/MacOS/*) printf '%s' "${exe%%/Contents/MacOS/*}"; return 0 ;;
        esac
    done
    return 1
}

APP="${1:-}"
[ -n "$APP" ] || APP=$(running_dolphin_app || true)
if [ -z "$APP" ]; then
    for candidate in \
        "$HOME/Library/Application Support/Slippi Launcher/netplay/Slippi Dolphin.app" \
        "$HOME/Library/Application Support/Slippi Launcher/netplay-beta/Slippi_Dolphin.app" \
        "$HOME/Library/Application Support/Slippi Launcher/playback/Slippi Dolphin.app" \
        "/Applications/Dolphin.app" \
        "$HOME/Applications/Dolphin.app"; do
        [ -d "$candidate" ] && APP="$candidate" && break
    done
fi
if [ -z "$APP" ] || [ ! -d "$APP" ]; then
    echo "usage: $0 /path/to/Dolphin.app" >&2
    exit 1
fi

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
entitlements="$work/entitlements.plist"

if ! codesign -d --entitlements :- "$APP" >"$entitlements" 2>/dev/null || [ ! -s "$entitlements" ]; then
    cat >"$entitlements" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict/></plist>
PLIST
fi

# PlistBuddy rather than plutil: plutil reads dots in a key path as nesting.
# disable-library-validation matters: the bundle loads dylibs signed by another
# team, which hardened runtime refuses once we re-sign the executable ad-hoc.
for key in com.apple.security.get-task-allow com.apple.security.cs.disable-library-validation; do
    /usr/libexec/PlistBuddy -c "Delete :$key" "$entitlements" 2>/dev/null || true
    /usr/libexec/PlistBuddy -c "Add :$key bool true" "$entitlements"
done

if ! codesign --force --options runtime --entitlements "$entitlements" --sign - "$APP" 2>"$work/codesign.err"; then
    if grep -q "Operation not permitted" "$work/codesign.err"; then
        echo "macOS blocked the signature change (App Management protection)." >&2
        echo "Allow your terminal under System Settings > Privacy & Security > App Management," >&2
        echo "then run this script again." >&2
    else
        cat "$work/codesign.err" >&2
    fi
    exit 1
fi

echo "==> $(basename "$APP") is now debuggable, with entitlements:"
codesign -d --entitlements :- "$APP" 2>/dev/null | tr '<' '\n' | sed -n 's/^key>\(.*\)/    \1/p'
echo "==> Restart Dolphin/Slippi, then press Start Stream in Dolphin mode."
