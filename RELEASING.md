# Orca Input Viewer — Signing + Auto-Updates (Windows + macOS + Linux)

This repo is set up to ship **signed** Windows installers and support **in-app auto-updates** via **GitHub Releases**.

## What’s implemented in code/CI

- **Tauri updater** enabled (checks GitHub Releases `latest.json` and shows release notes in-app).
- **Windows code signing** during the release workflow using a certificate imported into the runner.
- A **Release workflow** that:
  - builds the Windows installer,
  - signs it,
  - generates updater artifacts (`latest.json`, signatures),
  - publishes a GitHub Release.

## One-time setup (things you must do outside code)

### 1) Get a Windows code signing certificate

To remove “Unknown publisher” warnings, you need an Authenticode code signing certificate.

- Recommended: **EV Code Signing** (best SmartScreen reputation behavior).
- Standard code signing cert also works but can still trigger SmartScreen warnings until reputation builds.

#### If you want cloud signing (recommended for EV)

Most EV offerings use an HSM-backed private key that is **non-exportable**, so you will *not* get a `.pfx` you can base64 and store in GitHub Secrets.

In that case, pick a CA + “cloud/remote signing” product (examples: DigiCert KeyLocker / Signing Manager, Sectigo Code Signing Service, GlobalSign Atlas, SSL.com eSigner), then follow their docs to:

- Create a **non-interactive** signing identity for CI (API key / service account).
- Use their **signing CLI** (or Windows signer client) in GitHub Actions.

Our current `release.yml` is written for the **PFX** approach. When you choose a cloud signing provider, we should update `OrcaInputViewer/.github/workflows/release.yml` to use that provider’s signing tool instead of importing a PFX.

#### If you are using a PFX (standard cert, or exportable key)

You’ll end up with a **PFX** file and its password, and the existing workflow can import it into the runner certificate store and sign via `signtool`.

### 2) Generate Tauri updater signing keys (public + private)

Tauri’s updater verifies downloads using a signing keypair. The **public key** is committed in the app config; the **private key** is stored as a GitHub secret and used in CI to sign release artifacts.

From `OrcaInputViewer/src-tauri`:

```sh
cargo tauri signer generate
```

Notes:
- Use the password prompt to set a private key password (you’ll store it in GitHub Secrets).
- The command outputs a **public key** and a **private key** (or writes the private key to a file if you use `--write-keys`).

Update the app config:
- Edit `OrcaInputViewer/src-tauri/tauri.conf.json`
- Replace:
  - `plugins.updater.pubkey` = `REPLACE_ME_TAURI_UPDATER_PUBLIC_KEY`
  - with the **public key** printed by the signer.

### 3) Add GitHub repo secrets

In GitHub: **Settings → Secrets and variables → Actions → New repository secret**

Add these:

- `WINDOWS_CERT_PFX_B64`  
  Base64 of your `.pfx` file.
  - macOS/Linux: `base64 -i cert.pfx | pbcopy` (or remove `pbcopy`)
  - Windows PowerShell: `[Convert]::ToBase64String([IO.File]::ReadAllBytes("cert.pfx"))`

- `WINDOWS_CERT_PASSWORD`  
  Password for the `.pfx`.

- `TAURI_SIGNING_PRIVATE_KEY`  
  The **private key** output by `cargo tauri signer generate` (or the contents of the private key file).

- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`  
  The private key password you chose during signer generation.

## How to make a release

### 1) Bump version

These must stay in sync:

- `OrcaInputViewer/src-tauri/tauri.conf.json` → `version`
- `OrcaInputViewer/src-tauri/Cargo.toml` → `[package].version`

### 2) Tag and push

Create a git tag that matches your version (recommended format: `vX.Y.Z`):

```sh
git tag v0.1.1
git push origin v0.1.1
```

This triggers `OrcaInputViewer/.github/workflows/release.yml`.

### 3) Write good release notes

The in-app “changelog” display comes from the update metadata generated for the release.
Use GitHub Release notes (the workflow uses `generateReleaseNotes: true` by default, but you can edit the release text after).

## How updates work in-app

- On launch, the app checks:
  - `https://github.com/theorcadotgg/orca-obs-viewer/releases/latest/download/latest.json`
- If a newer version exists:
  - it shows a modal with the release notes,
  - clicking **Update & Restart** downloads and launches the installer (Windows updater exits the app and runs the installer).

## Debugging checklist

- “Updater not available in this build”
  - Ensure the updater plugin is enabled and the build includes it.
  - Ensure `OrcaInputViewer/src-tauri/capabilities/default.json` includes `updater:default`.

- “Signature verification failed”
  - Make sure `plugins.updater.pubkey` in `tauri.conf.json` matches the public key for the private key in `TAURI_SIGNING_PRIVATE_KEY`.
  - Make sure you are not post-processing (modifying) the installer after signatures are generated.

- Still seeing SmartScreen warnings
  - Standard certs may warn until reputation builds.
  - EV certs usually reduce friction significantly.

## macOS

The Windows CI workflow does not cover macOS. Everything below runs on a Mac.

### Build

```sh
cd OrcaInputViewer
cargo tauri build --bundles app      # or omit --bundles for .app + .dmg
open "src-tauri/target/release/bundle/macos/Orca Input Viewer.app"
```

Requires Rust plus the Xcode command line tools. `rusb` builds libusb from source
(the `vendored` feature), so Homebrew libusb is not needed and the bundle ships with
no library dependencies outside `/usr/lib` and `/System/Library`.

macOS builds are Apple Silicon only (`aarch64-apple-darwin`); there is no Intel build.

### Signing

Local builds are ad-hoc signed (`bundle.macOS.signingIdentity: "-"`) with
`src-tauri/entitlements.plist`, which grants `com.apple.security.cs.debugger`.
That entitlement is what allows the viewer to call `task_for_pid()` and read the
emulator's RAM — without it macOS returns `KERN_FAILURE (5)`.

For a distributable build, override the identity and notarize:

```sh
export APPLE_SIGNING_IDENTITY="Developer ID Application: <Name> (<TEAMID>)"
export APPLE_ID="you@example.com"
export APPLE_PASSWORD="app-specific-password"
export APPLE_TEAM_ID="<TEAMID>"
cargo tauri build
```

Keep the debug entitlement in the shipped build: it is the only way Dolphin mode
works on macOS. Note that macOS TCC records "App Management" permission per code
identity — an ad-hoc signature changes on every build, so users must re-approve it
after each update, while a Developer ID signature persists.

### macOS prerequisites for Dolphin mode

macOS only hands out task ports to entitled processes, so two things are needed:

1. the viewer carries `com.apple.security.cs.debugger` (done by the build config), and
2. the emulator bundle carries `com.apple.security.get-task-allow`.

Notarized Dolphin/Slippi builds do not carry (2), so one of these:

- In the app: press **Enable macOS Access**. It re-signs every Dolphin/Slippi install
  it finds (the running one, the copies Slippi Launcher manages under
  `~/Library/Application Support/Slippi Launcher/{netplay,netplay-beta,playback}`,
  and `/Applications/Dolphin.app`), keeping the entitlements those builds rely on
  (`com.apple.security.cs.allow-jit` is what makes Dolphin run on Apple Silicon) and
  adding `get-task-allow` plus `disable-library-validation`. Restart Dolphin afterwards.
- Or from a terminal: `scripts/macos-enable-dolphin-access.sh [path/to/Dolphin.app]`.

Both need the modifying app to hold the **App Management** permission
(System Settings → Privacy & Security → App Management). The app prompts for it on
first use; a terminal needs to be enabled there manually. Slippi Launcher replaces
its Dolphin on update, so re-run this after an update.

## Linux

Runs on x86_64 (the CI target is `x86_64-unknown-linux-gnu`); the AppImage is the
updatable artifact, since Tauri's updater has no deb/rpm path.

### Build

Install Rust plus the Tauri system dependencies (Arch/Omarchy names; `patchelf`
and `xdg-utils` are what the AppImage bundler's linuxdeploy step needs):

```sh
sudo pacman -S --needed base-devel webkit2gtk-4.1 librsvg pkgconf patchelf xdg-utils
cargo install tauri-cli --version '^2' --locked
cd OrcaInputViewer
cargo tauri build --bundles deb,appimage
```

On Debian/Ubuntu the build dependencies are
`libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf xdg-utils`.
Build the AppImage on the oldest distribution you intend to support — the bundle
links against that system's WebKitGTK.

A `tauri-cli` newer than the `tauri` crate in `Cargo.lock` logs
`Failed to add bundler type to the binary: __TAURI_BUNDLE_TYPE variable not found`.
It is harmless: the marker only exists in matching versions, and the updater
decides deb vs AppImage from the package itself rather than from that marker.

### Device access

`scripts/linux-enable-device-access.sh` installs the udev rules for the GameCube
adapter and the RP2040 config port, then reports whether Dolphin mode can read the
emulator's memory. Two independent things have to be true:

1. **udev rules** — without them the adapter and config interface are root-owned and
   the viewer cannot open them. The rules grant the logged-in user access
   (`TAG+="uaccess"`); replug the devices afterwards.
2. **ptrace access** — Dolphin mode reads the emulator's emulated RAM out of
   `/proc/<pid>/mem`, which the kernel gates on `kernel.yama.ptrace_scope`. With the
   default of `1`, the first Start Stream that finds a Dolphin process fails, and the
   viewer offers to fix it itself: it asks polkit (via `pkexec`) for one authorised
   change, which writes the drop-in file below and applies it, then restarts the
   stream. Declining is not sticky — the next Start Stream asks again.

   The prompt needs a polkit authentication agent. GNOME, KDE and XFCE ship one;
   minimal Wayland sessions (Hyprland, sway) do not, and there the app falls back to
   printing the manual commands. On Arch that agent is `hyprpolkitagent` or
   `polkit-gnome`.

   ```sh
   # AppImage: allow ptrace system-wide (persist in /etc/sysctl.d/99-orca-input-viewer.conf)
   sudo sysctl -w kernel.yama.ptrace_scope=0
   # or .deb/.rpm: grant the capability to just this binary
   sudo setcap cap_sys_ptrace=eip "$(command -v orca_input_viewer)"
   ```

   The capability route cannot work for an AppImage: it mounts read-only at a fresh
   `/tmp/.mount_*` path on every launch, so no stable file exists to carry it.

   The same authorised step installs the udev rules when they are missing, so a
   Dolphin-mode user gets adapter access in the same breath. Standalone (USB adapter)
   mode needs no ptrace permission at all, only those udev rules.

### Wayland

The overlay window appears, but Wayland has no always-on-top hint, so Tauri's
`always_on_top` and window positioning silently do nothing there
([tauri#14913](https://github.com/tauri-apps/tauri/issues/14913),
[tauri#13121](https://github.com/tauri-apps/tauri/issues/13121)) — the **Pin** button
in the overlay titlebar and the **Pin** toggle have no effect under Hyprland and
friends. Use **OBS browser source** for streaming: it is server-rendered to a
browser, so it is unaffected. Transparency (`transparent: true` on the overlay
window) also interacts badly with WebKitGTK's DMABUF renderer on NVIDIA
([tauri#14924](https://github.com/tauri-apps/tauri/issues/14924)); if the overlay
crashes or renders as black boxes, run with `WEBKIT_DISABLE_DMABUF_RENDERER=1`
(that disables transparency too, so the browser source is the better answer).

### CI (Windows + macOS + Linux)

`.github/workflows/release.yml` runs on `v*` tags (or manually):

1. `prepare` creates a **draft** GitHub release for the tag.
2. `windows`, `macos` and `linux` build, sign and upload into that draft (macOS is
   `aarch64-apple-darwin`, Developer ID signed and notarized; Linux is
   `x86_64-unknown-linux-gnu`, producing a deb, an rpm and the signed AppImage that
   the updater uses). The Linux job runs `cargo test` first, which exercises the
   Dolphin memory reader against a real process. Tauri Action merges each
   job's entry into one `latest.json`.
3. `publish` publishes the draft, so `releases/latest` — the updater endpoint — only ever
   points at a complete release.

Secrets to add under **Settings → Secrets and variables → Actions**:

| Secret | Platform | Value |
| --- | --- | --- |
| `WINDOWS_CERT_PFX_B64`, `WINDOWS_CERT_PASSWORD` | Windows | optional — installers ship unsigned (with a CI warning) until these are set |
| `APPLE_CERTIFICATE` | macOS | base64 of the exported `Developer ID Application` `.p12` |
| `APPLE_CERTIFICATE_PASSWORD` | macOS | password chosen when exporting that `.p12` |
| `KEYCHAIN_PASSWORD` | macOS | any random string; password of the throwaway CI keychain |
| `APPLE_ID` | macOS | Apple ID email |
| `APPLE_PASSWORD` | macOS | **app-specific** password from appleid.apple.com, not the account password |
| `APPLE_TEAM_ID` | macOS | team ID (`9987498A4X`) |
| `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | all | updater signing key (the Linux entry points at the AppImage signature) |

Exporting the certificate for `APPLE_CERTIFICATE`: Keychain Access → *My Certificates* →
expand “Developer ID Application: …” → right-click the key → *Export* → save as
`certificate.p12` with a password, then

```sh
openssl base64 -A -in certificate.p12 -out certificate-base64.txt
```

and paste the contents of that file into the secret.

The macOS job signs with the identity hard-coded in the workflow (update it if the
certificate is renewed) and notarizes whenever `APPLE_ID`/`APPLE_PASSWORD`/`APPLE_TEAM_ID`
are present. Release builds keep `entitlements.plist`, because the
`com.apple.security.cs.debugger` entitlement is what makes Dolphin mode work. If Apple's
notary ever rejects that entitlement, remove the three notarization secrets from the macOS
job — the DMG is then signed but not notarized.

### Auto-updates

`plugins.updater.pubkey` is still a placeholder, and both platform jobs refuse to release
while it is — replace it with the public key from `cargo tauri signer generate` (see above),
then the workflow produces the updater artifacts (`latest.json` plus `.app.tar.gz`/installer
signatures) and the in-app updater starts working.
