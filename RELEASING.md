# Orca Input Viewer — Signing + Auto-Updates (Windows)

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
