//! macOS-only helper: make the local Dolphin/Slippi bundle debuggable.
//!
//! macOS hands out task ports (needed to read the emulator's RAM) only to
//! entitled processes. The viewer side is covered by
//! `src-tauri/entitlements.plist` (`com.apple.security.cs.debugger`); the
//! emulator bundle must carry `com.apple.security.get-task-allow`, which
//! notarized Dolphin/Slippi builds do not ship with. So re-sign the local
//! bundle in place, keeping every entitlement the build already relies on
//! (`com.apple.security.cs.allow-jit` is what makes Dolphin run on Apple
//! Silicon). Slippi Launcher replaces its Dolphin on update, hence the retry.

use crate::dolphin::is_dolphin_process_name;
use std::path::{Path, PathBuf};
use std::process::Command;
use sysinfo::{ProcessRefreshKind, RefreshKind, System, UpdateKind};

const GET_TASK_ALLOW: &str = "com.apple.security.get-task-allow";
/// Needed because we re-sign the executable ad-hoc while the bundle loads
/// dylibs signed by another team; hardened runtime rejects those otherwise.
const DISABLE_LIBRARY_VALIDATION: &str = "com.apple.security.cs.disable-library-validation";

pub fn enable_dolphin_debug_access() -> Result<String, String> {
    let bundles = dolphin_bundles();
    if bundles.is_empty() {
        return Err(
            "No Dolphin or Slippi install found. Pass a path to \
             scripts/macos-enable-dolphin-access.sh instead."
                .to_string(),
        );
    }

    let mut signed = Vec::new();
    let mut failed = Vec::new();
    for bundle in &bundles {
        if read_entitlements(bundle)?.contains(GET_TASK_ALLOW) {
            continue;
        }
        match sign_bundle(bundle) {
            Ok(()) => signed.push(display(bundle)),
            Err(error) => failed.push(error),
        }
    }

    let mut message = if signed.is_empty() {
        format!(
            "Dolphin already allows debugging ({}). Restart it, then press Start Stream.",
            bundles
                .iter()
                .map(|bundle| display(bundle))
                .collect::<Vec<_>>()
                .join(", ")
        )
    } else {
        format!(
            "Signed {}. Restart Dolphin, then press Start Stream.",
            signed.join(", ")
        )
    };

    if failed.is_empty() {
        Ok(message)
    } else {
        message.push(' ');
        message.push_str(&failed.join(" "));
        Err(message)
    }
}

fn sign_bundle(bundle: &Path) -> Result<(), String> {
    let entitlements = read_entitlements(bundle)?;
    let merged = add_missing_entitlements(&entitlements, &[GET_TASK_ALLOW, DISABLE_LIBRARY_VALIDATION]);

    let temporary = std::env::temp_dir().join(format!("orca-entitlements-{}.plist", std::process::id()));
    std::fs::write(&temporary, merged).map_err(|e| format!("Could not write entitlements: {e}"))?;

    let output = Command::new("codesign")
        .args([
            "--force",
            "--options",
            "runtime",
            "--entitlements",
            &temporary.to_string_lossy(),
            "--sign",
            "-",
            &bundle.to_string_lossy(),
        ])
        .output()
        .map_err(|e| format!("Could not run codesign: {e}"))?;
    let _ = std::fs::remove_file(&temporary);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(if stderr.contains("Operation not permitted") {
            "macOS blocked the signature change. Open System Settings > Privacy & Security > \
             App Management, allow \"Orca Input Viewer\", then try again."
                .to_string()
        } else {
            format!("codesign failed for {}: {}", display(bundle), stderr.trim())
        });
    }

    if !read_entitlements(bundle)?.contains(GET_TASK_ALLOW) {
        return Err(format!(
            "Signature of {} did not take; run scripts/macos-enable-dolphin-access.sh from a \
             Terminal with App Management permission.",
            display(bundle)
        ));
    }

    Ok(())
}

/// Emulator bundles to sign: the running instance first, then every install
/// Slippi Launcher manages (netplay = mainline, beta, playback) and a stock
/// Dolphin. Slippi users may launch any of them, so cover them all.
fn dolphin_bundles() -> Vec<PathBuf> {
    let home = std::env::var("HOME").unwrap_or_default();
    let candidates = [
        format!("{home}/Library/Application Support/Slippi Launcher/netplay/Slippi Dolphin.app"),
        format!("{home}/Library/Application Support/Slippi Launcher/netplay-beta/Slippi_Dolphin.app"),
        format!("{home}/Library/Application Support/Slippi Launcher/playback/Slippi Dolphin.app"),
        "/Applications/Dolphin.app".to_string(),
        format!("{home}/Applications/Dolphin.app"),
    ];

    running_dolphin_bundles()
        .into_iter()
        .chain(
            candidates
                .iter()
                .map(PathBuf::from)
                .filter(|path| path.exists()),
        )
        .fold(Vec::new(), |mut unique, bundle| {
            if !unique.contains(&bundle) {
                unique.push(bundle);
            }
            unique
        })
}

fn running_dolphin_bundles() -> Vec<PathBuf> {
    let system = System::new_with_specifics(
        RefreshKind::new()
            .with_processes(ProcessRefreshKind::new().with_exe(UpdateKind::Always)),
    );

    system
        .processes()
        .iter()
        .filter(|(_, process)| is_dolphin_process_name(&process.name().to_string()))
        .filter_map(|(_, process)| process.exe().and_then(bundle_containing))
        .collect()
}

/// `/path/Foo.app/Contents/MacOS/Foo` -> `/path/Foo.app`
fn bundle_containing(executable: &Path) -> Option<PathBuf> {
    executable
        .ancestors()
        .find(|path| path.extension().is_some_and(|extension| extension == "app"))
        .map(Path::to_path_buf)
}

fn read_entitlements(bundle: &Path) -> Result<String, String> {
    let output = Command::new("codesign")
        .args(["-d", "--entitlements", ":-", &bundle.to_string_lossy()])
        .output()
        .map_err(|e| format!("Could not run codesign: {e}"))?;

    let entitlements = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if entitlements.is_empty() {
        // Nothing to inherit: start from an empty dict.
        return Ok(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?><plist version=\"1.0\"><dict></dict></plist>"
                .to_string(),
        );
    }
    Ok(entitlements)
}

fn display(bundle: &Path) -> String {
    bundle
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| bundle.to_string_lossy().into_owned())
}

/// Add `<key>…</key><true/>` for every missing key, before the closing dict.
/// Duplicate keys make `codesign` fail with "duplicate dictionary key", so keys
/// the bundle already declares are left untouched.
fn add_missing_entitlements(entitlements: &str, keys: &[&str]) -> String {
    let mut added = String::new();
    for key in keys {
        if !entitlements.contains(key) {
            added.push_str(&format!("<key>{key}</key><true/>"));
        }
    }

    let (head, tail) = match entitlements.rfind("</dict>") {
        Some(index) => entitlements.split_at(index),
        None => (entitlements, ""),
    };
    format!("{head}{added}{tail}")
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXISTING: &str = "<?xml version=\"1.0\"?><plist version=\"1.0\"><dict>\
        <key>com.apple.security.cs.allow-jit</key><true/></dict></plist>";

    #[test]
    fn adds_missing_entitlements_and_keeps_existing_ones() {
        let merged = add_missing_entitlements(EXISTING, &[GET_TASK_ALLOW, DISABLE_LIBRARY_VALIDATION]);

        assert!(merged.contains(&format!("<key>{GET_TASK_ALLOW}</key><true/>")));
        assert!(merged.contains(&format!("<key>{DISABLE_LIBRARY_VALIDATION}</key><true/>")));
        assert!(merged.contains("com.apple.security.cs.allow-jit"));
        assert!(merged.ends_with("</dict></plist>"));
    }

    #[test]
    fn does_not_duplicate_a_key_that_is_already_present() {
        let merged = add_missing_entitlements(EXISTING, &[DISABLE_LIBRARY_VALIDATION]);
        let with_existing = add_missing_entitlements(&merged, &[DISABLE_LIBRARY_VALIDATION]);

        assert_eq!(with_existing.matches(DISABLE_LIBRARY_VALIDATION).count(), 1);
    }

    #[test]
    fn finds_the_bundle_containing_an_executable() {
        let executable = PathBuf::from("/Applications/Dolphin.app/Contents/MacOS/Dolphin");

        assert_eq!(
            bundle_containing(&executable),
            Some(PathBuf::from("/Applications/Dolphin.app"))
        );
        assert_eq!(bundle_containing(Path::new("/usr/bin/codesign")), None);
    }
}
