//! Dolphin memory reading for controller input
//!
//! Reads controller state directly from Dolphin's emulated GameCube RAM,
//! allowing the input viewer to work while Dolphin has the USB adapter claimed.

use crate::InputReport;
use sysinfo::{ProcessRefreshKind, RefreshKind, System};

/// Process names to search for Dolphin/Slippi
const DOLPHIN_PROCESS_NAMES: &[&str] = &[
    "Dolphin.exe",
    "dolphin.exe",
    "Slippi Dolphin.exe",
    "Slippi_Dolphin.exe",
    "DolphinWx.exe",
    "DolphinQt.exe",
    // Unix names
    "dolphin-emu",
    "slippi-dolphin",
    "Slippi Dolphin",
];

#[derive(Debug)]
pub struct DolphinProcess {
    pub pid: u32,
    pub name: String,
}

/// Find a running Dolphin or Slippi process
pub fn find_dolphin_process() -> Option<DolphinProcess> {
    let system = System::new_with_specifics(
        RefreshKind::new().with_processes(ProcessRefreshKind::new()),
    );

    for (pid, process) in system.processes() {
        let name = process.name().to_string();
        for &dolphin_name in DOLPHIN_PROCESS_NAMES {
            if name.contains(dolphin_name) || name == dolphin_name {
                return Some(DolphinProcess {
                    pid: pid.as_u32(),
                    name,
                });
            }
        }
    }
    None
}

#[cfg(target_os = "windows")]
mod windows {
    use super::*;
    use crate::{Axes, Buttons, PortReport};
    use std::ptr;
    use winapi::shared::minwindef::{DWORD, FALSE, LPCVOID};
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::memoryapi::{ReadProcessMemory, VirtualQueryEx};
    use winapi::um::processthreadsapi::OpenProcess;
    use winapi::um::winnt::{
        HANDLE, MEMORY_BASIC_INFORMATION, MEM_MAPPED, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
    };

    /// Memory addresses for Melee NTSC v1.0 controller data
    const MELEE_CONTROLLER_BASE: u32 = 0x804BFE2C;
    const CONTROLLER_STRUCT_SIZE: u32 = 0x44;

    /// Button bit flags in Melee's controller struct
    const BTN_A: u32 = 0x0100;
    const BTN_B: u32 = 0x0200;
    const BTN_X: u32 = 0x0400;
    const BTN_Y: u32 = 0x0800;
    const BTN_START: u32 = 0x1000;
    const BTN_DPAD_LEFT: u32 = 0x0001;
    const BTN_DPAD_RIGHT: u32 = 0x0002;
    const BTN_DPAD_DOWN: u32 = 0x0004;
    const BTN_DPAD_UP: u32 = 0x0008;
    const BTN_Z: u32 = 0x0010;
    const BTN_R: u32 = 0x0020;
    const BTN_L: u32 = 0x0040;

    fn default_buttons() -> Buttons {
        Buttons {
            a: false,
            b: false,
            x: false,
            y: false,
            z: false,
            l: false,
            r: false,
            start: false,
            dpad_up: false,
            dpad_down: false,
            dpad_left: false,
            dpad_right: false,
        }
    }

    fn default_axes() -> Axes {
        Axes {
            stick_x: 0.0,
            stick_y: 0.0,
            substick_x: 0.0,
            substick_y: 0.0,
            trigger_l: 0.0,
            trigger_r: 0.0,
        }
    }

    pub struct DolphinReader {
        handle: HANDLE,
        gc_ram_base: usize,
    }

    impl DolphinReader {
        pub fn new(process: &DolphinProcess) -> Result<Self, String> {
            unsafe {
                let handle = OpenProcess(
                    PROCESS_VM_READ | PROCESS_QUERY_INFORMATION,
                    FALSE,
                    process.pid as DWORD,
                );
                if handle.is_null() {
                    return Err("Failed to open Dolphin process".to_string());
                }

                // Find the GameCube RAM region (32MB MEM_MAPPED region)
                match find_gc_ram_base(handle) {
                    Some(base) => Ok(DolphinReader {
                        handle,
                        gc_ram_base: base,
                    }),
                    None => {
                        CloseHandle(handle);
                        Err("Could not find GameCube RAM in Dolphin memory".to_string())
                    }
                }
            }
        }

        pub fn read_controller_state(&self) -> Result<InputReport, String> {
            let mut ports = Vec::with_capacity(4);

            for port in 0..4u8 {
                let controller_addr = MELEE_CONTROLLER_BASE + (port as u32 * CONTROLLER_STRUCT_SIZE);
                let host_addr = self.gc_ram_base + (controller_addr & 0x7FFFFFFF) as usize;

                // Read controller struct (48 bytes covers all fields we need)
                let mut buf = [0u8; 48];
                if !self.read_memory(host_addr, &mut buf) {
                    // If we can't read, assume disconnected
                    ports.push(PortReport {
                        port,
                        connected: false,
                        buttons: default_buttons(),
                        axes: default_axes(),
                    });
                    continue;
                }

                // Parse button state (offset 0x00, big-endian u32)
                let buttons_raw = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);

                // Parse analog values
                let trigger_l = buf[0x1C];
                let trigger_r = buf[0x1D];

                // Joystick values are floats at offsets 0x20, 0x24, 0x28, 0x2C
                let stick_x = f32::from_be_bytes([buf[0x20], buf[0x21], buf[0x22], buf[0x23]]);
                let stick_y = f32::from_be_bytes([buf[0x24], buf[0x25], buf[0x26], buf[0x27]]);
                let substick_x = f32::from_be_bytes([buf[0x28], buf[0x29], buf[0x2A], buf[0x2B]]);
                let substick_y = f32::from_be_bytes([buf[0x2C], buf[0x2D], buf[0x2E], buf[0x2F]]);

                // Check if controller is connected (non-zero button state or stick movement)
                let connected = buttons_raw != 0
                    || trigger_l > 0
                    || trigger_r > 0
                    || stick_x.abs() > 0.01
                    || stick_y.abs() > 0.01
                    || substick_x.abs() > 0.01
                    || substick_y.abs() > 0.01;

                let buttons = Buttons {
                    a: (buttons_raw & BTN_A) != 0,
                    b: (buttons_raw & BTN_B) != 0,
                    x: (buttons_raw & BTN_X) != 0,
                    y: (buttons_raw & BTN_Y) != 0,
                    z: (buttons_raw & BTN_Z) != 0,
                    l: (buttons_raw & BTN_L) != 0,
                    r: (buttons_raw & BTN_R) != 0,
                    start: (buttons_raw & BTN_START) != 0,
                    dpad_up: (buttons_raw & BTN_DPAD_UP) != 0,
                    dpad_down: (buttons_raw & BTN_DPAD_DOWN) != 0,
                    dpad_left: (buttons_raw & BTN_DPAD_LEFT) != 0,
                    dpad_right: (buttons_raw & BTN_DPAD_RIGHT) != 0,
                };

                let axes = Axes {
                    stick_x,
                    stick_y,
                    substick_x,
                    substick_y,
                    trigger_l: trigger_l as f32 / 140.0, // Melee uses 0-140 range
                    trigger_r: trigger_r as f32 / 140.0,
                };

                ports.push(PortReport {
                    port,
                    connected,
                    buttons,
                    axes,
                });
            }

            Ok(InputReport { ports })
        }

        fn read_memory(&self, addr: usize, buf: &mut [u8]) -> bool {
            unsafe {
                let mut bytes_read = 0usize;
                ReadProcessMemory(
                    self.handle,
                    addr as LPCVOID,
                    buf.as_mut_ptr() as *mut _,
                    buf.len(),
                    &mut bytes_read,
                ) != FALSE
                    && bytes_read == buf.len()
            }
        }
    }

    impl Drop for DolphinReader {
        fn drop(&mut self) {
            unsafe {
                CloseHandle(self.handle);
            }
        }
    }

    /// Find the 32MB GameCube RAM region in Dolphin's memory
    unsafe fn find_gc_ram_base(handle: HANDLE) -> Option<usize> {
        let mut addr: usize = 0;
        let mut mbi: MEMORY_BASIC_INFORMATION = std::mem::zeroed();

        // Target size: 32MB (GameCube RAM)
        const GC_RAM_SIZE: usize = 32 * 1024 * 1024;

        while VirtualQueryEx(
            handle,
            addr as LPCVOID,
            &mut mbi,
            std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
        ) != 0
        {
            // Look for a MEM_MAPPED region of exactly 32MB
            if mbi.Type == MEM_MAPPED && mbi.RegionSize == GC_RAM_SIZE {
                return Some(mbi.BaseAddress as usize);
            }

            // Move to next region
            addr = mbi.BaseAddress as usize + mbi.RegionSize;
            if addr == 0 {
                break;
            }
        }

        None
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;

    pub struct DolphinReader {
        _pid: u32,
    }

    impl DolphinReader {
        pub fn new(process: &DolphinProcess) -> Result<Self, String> {
            // macOS requires special entitlements or root access for task_for_pid
            // For now, return an error with instructions
            Err(format!(
                "Dolphin memory reading on macOS requires special permissions. \
                 Process '{}' (pid {}) found, but memory access is not yet supported. \
                 Please use USB mode with Dolphin closed.",
                process.name, process.pid
            ))
        }

        pub fn read_controller_state(&self) -> Result<InputReport, String> {
            Err("macOS memory reading not implemented".to_string())
        }
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use super::*;

    pub struct DolphinReader {
        _pid: u32,
    }

    impl DolphinReader {
        pub fn new(process: &DolphinProcess) -> Result<Self, String> {
            // Linux requires ptrace capability or reading /proc/{pid}/mem
            Err(format!(
                "Dolphin memory reading on Linux requires ptrace capability. \
                 Process '{}' (pid {}) found. Try: sudo setcap cap_sys_ptrace=eip <app>",
                process.name, process.pid
            ))
        }

        pub fn read_controller_state(&self) -> Result<InputReport, String> {
            Err("Linux memory reading not implemented".to_string())
        }
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
mod unsupported {
    use super::*;

    pub struct DolphinReader;

    impl DolphinReader {
        pub fn new(_process: &DolphinProcess) -> Result<Self, String> {
            Err("Dolphin memory reading not supported on this platform".to_string())
        }

        pub fn read_controller_state(&self) -> Result<InputReport, String> {
            Err("Not supported".to_string())
        }
    }
}

#[cfg(target_os = "windows")]
pub use windows::DolphinReader;

#[cfg(target_os = "macos")]
pub use macos::DolphinReader;

#[cfg(target_os = "linux")]
pub use linux::DolphinReader;

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub use unsupported::DolphinReader;
