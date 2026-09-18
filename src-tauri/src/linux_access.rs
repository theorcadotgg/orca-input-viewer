//! Linux-only helper: grant the viewer the access Dolphin mode and the USB
//! adapter need.
//!
//! Two independent things are root-owned on a stock install:
//!
//!  * reading another process's memory is a ptrace operation, and
//!    `kernel.yama.ptrace_scope` defaults to 1, which permits tracing only
//!    descendants - so the viewer cannot read Slippi's emulated RAM;
//!  * the GameCube adapter and Orca's config interface are USB and serial
//!    devices that root owns unless a udev rule says otherwise.
//!
//! Neither can be granted to the binary itself: an AppImage mounts read-only at
//! a fresh path on every launch, so there is no stable file to hang a
//! capability on. Instead the app asks polkit for one authorised change, which
//! writes a sysctl drop-in and the udev rules and reloads both.
//!
//! The tools used here (`pkexec`, `/bin/sh`, `sysctl`, `udevadm`) are the
//! standard ones on systemd/udev desktops, so this is distribution-neutral.
//! Where polkit or its authentication agent is missing the error says so and
//! gives the commands to run by hand.

use crate::{ADAPTER_PID, ADAPTER_VID, CONFIG_PID, CONFIG_VID};
use std::process::Command;

const PTRACE_SCOPE: &str = "/proc/sys/kernel/yama/ptrace_scope";
const SYSCTL_FILE: &str = "/etc/sysctl.d/99-orca-input-viewer.conf";
const UDEV_FILE: &str = "/etc/udev/rules.d/51-orca-input-viewer.rules";

/// Which of the two permissions still has to be granted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Missing {
    pub ptrace: bool,
    pub udev: bool,
}

impl Missing {
    fn any(self) -> bool {
        self.ptrace || self.udev
    }
}

/// `ptrace_scope` is `None` when the kernel has no Yama at all, in which case
/// there is nothing to relax. Any non-zero scope blocks a non-descendant.
pub fn missing_access(ptrace_scope: Option<&str>, udev_rules: bool) -> Missing {
    Missing {
        ptrace: ptrace_scope.is_some_and(|scope| scope.trim() != "0"),
        udev: !udev_rules,
    }
}

fn read_ptrace_scope() -> Option<String> {
    std::fs::read_to_string(PTRACE_SCOPE).ok()
}

/// True when the rules file is present *and* ours: a same-named file from
/// another tool must not make us skip the step.
fn udev_rules_installed() -> bool {
    std::fs::read_to_string(UDEV_FILE).is_ok_and(|rules| {
        rules.contains(&format!("{:04x}", ADAPTER_PID)) && rules.contains(&format!("{:04x}", CONFIG_PID))
    })
}

/// Both changes in one script, so the user sees a single authorisation prompt.
fn access_script(missing: Missing) -> String {
    let mut script = String::from("set -e\n");

    if missing.ptrace {
        script.push_str(&format!(
            "printf 'kernel.yama.ptrace_scope=0\\n' > {SYSCTL_FILE}\n\
             sysctl -q -w kernel.yama.ptrace_scope=0\n"
        ));
    }

    if missing.udev {
        script.push_str(&format!(
            "printf '%s\\n' \\
             'SUBSYSTEM==\"usb\", ATTRS{{idVendor}}==\"{adapter_vid:04x}\", ATTRS{{idProduct}}==\"{adapter_pid:04x}\", TAG+=\"uaccess\"' \\
             'SUBSYSTEM==\"tty\", ATTRS{{idVendor}}==\"{config_vid:04x}\", ATTRS{{idProduct}}==\"{config_pid:04x}\", TAG+=\"uaccess\"' \\
             > {UDEV_FILE}\n\
             udevadm control --reload-rules\n\
             udevadm trigger\n",
            adapter_vid = ADAPTER_VID,
            adapter_pid = ADAPTER_PID,
            config_vid = CONFIG_VID,
            config_pid = CONFIG_PID,
        ));
    }

    script
}

/// Ask polkit to apply whatever is missing. Returns a message for the UI.
pub fn enable_access() -> Result<String, String> {
    let missing = missing_access(read_ptrace_scope().as_deref(), udev_rules_installed());
    if !missing.any() {
        return Ok("Dolphin and adapter access are already enabled.".to_string());
    }

    let output = Command::new("pkexec")
        .arg("/bin/sh")
        .arg("-c")
        .arg(access_script(missing))
        .output()
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => manual_fallback("polkit's pkexec is not installed"),
            _ => manual_fallback(&format!("pkexec could not be run: {e}")),
        })?;

    if output.status.success() {
        return Ok(if missing.udev {
            "Access granted - replug the GameCube adapter and the Orca board.".to_string()
        } else {
            "Access granted.".to_string()
        });
    }

    Err(manual_fallback(&refusal_reason(
        output.status.code(),
        &String::from_utf8_lossy(&output.stderr),
    )))
}

/// polkit's own wording decides the advice, and its exit code covers the cases
/// where it says nothing useful: 127 when no agent could be reached (on a
/// desktop without one it prints "Error creating textual authentication agent"
/// past a failed tty lookup), 126 when the user dismissed the prompt.
fn refusal_reason(code: Option<i32>, stderr: &str) -> String {
    let detail = stderr
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .next_back()
        .unwrap_or("");

    if detail.contains("authentication agent") || code == Some(127) {
        "no polkit authentication agent is running - desktops normally provide one, \
         a minimal Wayland session needs one installed (hyprpolkitagent or polkit-gnome)"
            .to_string()
    } else if detail.contains("dismissed") || detail.contains("not authorized") || code == Some(126) {
        "the authorisation prompt was dismissed".to_string()
    } else if detail.is_empty() {
        "polkit refused the request".to_string()
    } else {
        detail.to_string()
    }
}

fn manual_fallback(reason: &str) -> String {
    format!(
        "Could not grant access automatically ({reason}). Dolphin mode needs \
         `sudo sysctl -w kernel.yama.ptrace_scope=0`; the adapter also needs the udev \
         rules from the Linux setup guide."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_access_reports_only_what_is_actually_missing() {
        // Scope 0 and the rules in place: a second Start Stream must not prompt.
        assert_eq!(missing_access(Some("0\n"), true), Missing { ptrace: false, udev: false });
        // Yama absent (kernel without it): nothing to relax, adapter rules still matter.
        assert_eq!(missing_access(None, false), Missing { ptrace: false, udev: true });
        // The default on Arch, Debian, Fedora and openSUSE.
        assert_eq!(missing_access(Some("1\n"), false), Missing { ptrace: true, udev: true });
        // Restricted scopes (2, 3) block a non-descendant just like 1.
        assert_eq!(missing_access(Some("2\n"), true), Missing { ptrace: true, udev: false });
    }

    #[test]
    fn the_script_only_touches_what_is_missing() {
        let both = access_script(Missing { ptrace: true, udev: true });
        assert!(both.contains(SYSCTL_FILE));
        assert!(both.contains(UDEV_FILE));
        assert!(both.contains("udevadm control --reload-rules"));

        let ptrace_only = access_script(Missing { ptrace: true, udev: false });
        assert!(!ptrace_only.contains(UDEV_FILE), "no udev rewrite: {ptrace_only}");

        let udev_only = access_script(Missing { ptrace: false, udev: true });
        assert!(!udev_only.contains("sysctl"), "no sysctl write: {udev_only}");
        assert!(udev_only.contains(UDEV_FILE));
    }

    #[test]
    fn refusal_reasons_come_from_polkits_wording() {
        // Captured from a real pkexec run on a box with polkit but no agent.
        let no_agent = "Error creating textual authentication agent: Error opening current \
                        controlling terminal for the process (`/dev/tty'): No such device or address";
        assert!(refusal_reason(Some(127), no_agent).contains("authentication agent"));
        assert!(refusal_reason(Some(127), "").contains("authentication agent"));
        assert!(
            refusal_reason(Some(1), "Error executing command as another user: No authentication agent found.").contains("authentication agent")
        );
        assert!(refusal_reason(Some(126), "").contains("dismissed"));
        assert!(refusal_reason(Some(1), "Error executing command as another user: Request dismissed").contains("dismissed"));
        assert_eq!(refusal_reason(Some(1), ""), "polkit refused the request");
        assert_eq!(refusal_reason(Some(1), "something else went wrong"), "something else went wrong");
    }

    /// The script is assembled by string formatting, quotes and all, and then
    /// handed to a root shell - so it has to parse.
    #[test]
    fn the_generated_script_is_valid_shell() {
        for missing in [
            Missing { ptrace: true, udev: true },
            Missing { ptrace: true, udev: false },
            Missing { ptrace: false, udev: true },
        ] {
            let script = access_script(missing);
            let status = Command::new("/bin/sh")
                .args(["-n", "-c", &script])
                .status()
                .expect("run sh -n");
            assert!(status.success(), "sh -n rejected {missing:?}:\n{script}");
        }
    }
}
