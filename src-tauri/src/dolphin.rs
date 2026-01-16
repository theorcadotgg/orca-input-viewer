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

    /// Memory addresses for Melee controller data (PADStatus structs)
    /// NTSC 1.02: 0x804C1FAC
    const MELEE_PAD_STATUS_BASE: u32 = 0x804C1FAC;
    const PAD_STATUS_SIZE: u32 = 0x0C; // 12 bytes per controller

    /// GameCube PADStatus button bit flags (big-endian u16)
    const PAD_BUTTON_LEFT: u16 = 0x0001;
    const PAD_BUTTON_RIGHT: u16 = 0x0002;
    const PAD_BUTTON_DOWN: u16 = 0x0004;
    const PAD_BUTTON_UP: u16 = 0x0008;
    const PAD_BUTTON_Z: u16 = 0x0010;
    const PAD_BUTTON_R: u16 = 0x0020;
    const PAD_BUTTON_L: u16 = 0x0040;
    const PAD_BUTTON_A: u16 = 0x0100;
    const PAD_BUTTON_B: u16 = 0x0200;
    const PAD_BUTTON_X: u16 = 0x0400;
    const PAD_BUTTON_Y: u16 = 0x0800;
    const PAD_BUTTON_START: u16 = 0x1000;

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
                let pad_addr = MELEE_PAD_STATUS_BASE + (port as u32 * PAD_STATUS_SIZE);
                // Mask off high bits to get offset within GC RAM (0x80000000 -> 0x00000000)
                let host_addr = self.gc_ram_base + (pad_addr & 0x01FFFFFF) as usize;

                // Read PADStatus struct (12 bytes)
                let mut buf = [0u8; 12];
                if !self.read_memory(host_addr, &mut buf) {
                    ports.push(PortReport {
                        port,
                        connected: false,
                        buttons: default_buttons(),
                        axes: default_axes(),
                    });
                    continue;
                }

                // Parse button state (offset 0x00, big-endian u16)
                let buttons_raw = u16::from_be_bytes([buf[0], buf[1]]);

                // Parse analog values - these are signed bytes (-128 to 127, 0 = center)
                let stick_x_raw = buf[2] as i8;
                let stick_y_raw = buf[3] as i8;
                let substick_x_raw = buf[4] as i8;
                let substick_y_raw = buf[5] as i8;
                let trigger_l = buf[6];
                let trigger_r = buf[7];

                // Convert to normalized floats (-1.0 to 1.0)
                let stick_x = stick_x_raw as f32 / 128.0;
                let stick_y = stick_y_raw as f32 / 128.0;
                let substick_x = substick_x_raw as f32 / 128.0;
                let substick_y = substick_y_raw as f32 / 128.0;

                // In Dolphin mode, if we can read memory, port 0 is always connected
                // Other ports we mark as connected only if they show activity
                let connected = if port == 0 {
                    true // Port 0 is always connected when reading from Dolphin
                } else {
                    buttons_raw != 0
                        || trigger_l > 0
                        || trigger_r > 0
                        || stick_x_raw != 0
                        || stick_y_raw != 0
                        || substick_x_raw != 0
                        || substick_y_raw != 0
                };

                let buttons = Buttons {
                    a: (buttons_raw & PAD_BUTTON_A) != 0,
                    b: (buttons_raw & PAD_BUTTON_B) != 0,
                    x: (buttons_raw & PAD_BUTTON_X) != 0,
                    y: (buttons_raw & PAD_BUTTON_Y) != 0,
                    z: (buttons_raw & PAD_BUTTON_Z) != 0,
                    l: (buttons_raw & PAD_BUTTON_L) != 0,
                    r: (buttons_raw & PAD_BUTTON_R) != 0,
                    start: (buttons_raw & PAD_BUTTON_START) != 0,
                    dpad_up: (buttons_raw & PAD_BUTTON_UP) != 0,
                    dpad_down: (buttons_raw & PAD_BUTTON_DOWN) != 0,
                    dpad_left: (buttons_raw & PAD_BUTTON_LEFT) != 0,
                    dpad_right: (buttons_raw & PAD_BUTTON_RIGHT) != 0,
                };

                let axes = Axes {
                    stick_x,
                    stick_y,
                    substick_x,
                    substick_y,
                    trigger_l: trigger_l as f32 / 255.0,
                    trigger_r: trigger_r as f32 / 255.0,
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
    use crate::{Axes, Buttons, PortReport};
    use mach2::kern_return::KERN_SUCCESS;
    use mach2::mach_types::task_t;
    use mach2::port::mach_port_t;
    use mach2::traps::task_for_pid;
    use mach2::vm::{mach_vm_read_overwrite, mach_vm_region};
    use mach2::vm_region::{vm_region_basic_info_data_64_t, VM_REGION_BASIC_INFO_64};
    use mach2::vm_types::{mach_vm_address_t, mach_vm_size_t};
    use std::mem;

    /// Memory addresses for Melee controller data (PADStatus structs)
    /// NTSC 1.02: 0x804C1FAC
    const MELEE_PAD_STATUS_BASE: u32 = 0x804C1FAC;
    const PAD_STATUS_SIZE: u32 = 0x0C; // 12 bytes per controller

    /// GameCube PADStatus button bit flags (big-endian u16)
    const PAD_BUTTON_LEFT: u16 = 0x0001;
    const PAD_BUTTON_RIGHT: u16 = 0x0002;
    const PAD_BUTTON_DOWN: u16 = 0x0004;
    const PAD_BUTTON_UP: u16 = 0x0008;
    const PAD_BUTTON_Z: u16 = 0x0010;
    const PAD_BUTTON_R: u16 = 0x0020;
    const PAD_BUTTON_L: u16 = 0x0040;
    const PAD_BUTTON_A: u16 = 0x0100;
    const PAD_BUTTON_B: u16 = 0x0200;
    const PAD_BUTTON_X: u16 = 0x0400;
    const PAD_BUTTON_Y: u16 = 0x0800;
    const PAD_BUTTON_START: u16 = 0x1000;

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
        task: task_t,
        gc_ram_base: mach_vm_address_t,
    }

    impl DolphinReader {
        pub fn new(process: &DolphinProcess) -> Result<Self, String> {
            unsafe {
                let mut task: task_t = 0;
                let kr = task_for_pid(
                    mach2::traps::mach_task_self(),
                    process.pid as i32,
                    &mut task,
                );

                if kr != KERN_SUCCESS {
                    return Err(format!(
                        "Failed to get task port for Dolphin (pid {}). Error: {}. \
                         You may need to run with sudo or grant accessibility permissions.",
                        process.pid, kr
                    ));
                }

                // Find the GameCube RAM region (32MB region)
                match find_gc_ram_base(task) {
                    Some(base) => Ok(DolphinReader {
                        task,
                        gc_ram_base: base,
                    }),
                    None => Err("Could not find GameCube RAM in Dolphin memory. \
                                 Make sure a game is running in Dolphin."
                        .to_string()),
                }
            }
        }

        pub fn read_controller_state(&self) -> Result<InputReport, String> {
            let mut ports = Vec::with_capacity(4);

            for port in 0..4u8 {
                let pad_addr = MELEE_PAD_STATUS_BASE + (port as u32 * PAD_STATUS_SIZE);
                // Mask off high bits to get offset within GC RAM (0x80000000 -> 0x00000000)
                let offset = (pad_addr & 0x01FFFFFF) as u64;
                let host_addr = self.gc_ram_base + offset;

                // Read PADStatus struct (12 bytes)
                // Layout: u16 buttons, i8 stickX, i8 stickY, i8 substickX, i8 substickY, u8 triggerL, u8 triggerR, u8 analogA, u8 analogB, i8 err, u8 padding
                let mut buf = [0u8; 12];
                if !self.read_memory(host_addr, &mut buf) {
                    ports.push(PortReport {
                        port,
                        connected: false,
                        buttons: default_buttons(),
                        axes: default_axes(),
                    });
                    continue;
                }

                // Parse button state (offset 0x00, big-endian u16)
                let buttons_raw = u16::from_be_bytes([buf[0], buf[1]]);

                // Parse analog values - these are signed bytes (-128 to 127, 0 = center)
                let stick_x_raw = buf[2] as i8;
                let stick_y_raw = buf[3] as i8;
                let substick_x_raw = buf[4] as i8;
                let substick_y_raw = buf[5] as i8;
                let trigger_l = buf[6];
                let trigger_r = buf[7];

                // Convert to normalized floats (-1.0 to 1.0)
                let stick_x = stick_x_raw as f32 / 128.0;
                let stick_y = stick_y_raw as f32 / 128.0;
                let substick_x = substick_x_raw as f32 / 128.0;
                let substick_y = substick_y_raw as f32 / 128.0;

                // In Dolphin mode, if we can read memory, port 0 is always connected
                // Other ports we mark as connected only if they show activity
                let connected = if port == 0 {
                    true // Port 0 is always connected when reading from Dolphin
                } else {
                    buttons_raw != 0
                        || trigger_l > 0
                        || trigger_r > 0
                        || stick_x_raw != 0
                        || stick_y_raw != 0
                        || substick_x_raw != 0
                        || substick_y_raw != 0
                };

                let buttons = Buttons {
                    a: (buttons_raw & PAD_BUTTON_A) != 0,
                    b: (buttons_raw & PAD_BUTTON_B) != 0,
                    x: (buttons_raw & PAD_BUTTON_X) != 0,
                    y: (buttons_raw & PAD_BUTTON_Y) != 0,
                    z: (buttons_raw & PAD_BUTTON_Z) != 0,
                    l: (buttons_raw & PAD_BUTTON_L) != 0,
                    r: (buttons_raw & PAD_BUTTON_R) != 0,
                    start: (buttons_raw & PAD_BUTTON_START) != 0,
                    dpad_up: (buttons_raw & PAD_BUTTON_UP) != 0,
                    dpad_down: (buttons_raw & PAD_BUTTON_DOWN) != 0,
                    dpad_left: (buttons_raw & PAD_BUTTON_LEFT) != 0,
                    dpad_right: (buttons_raw & PAD_BUTTON_RIGHT) != 0,
                };

                let axes = Axes {
                    stick_x,
                    stick_y,
                    substick_x,
                    substick_y,
                    trigger_l: trigger_l as f32 / 255.0,
                    trigger_r: trigger_r as f32 / 255.0,
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

        fn read_memory(&self, addr: mach_vm_address_t, buf: &mut [u8]) -> bool {
            unsafe {
                let mut size: mach_vm_size_t = 0;
                let kr = mach_vm_read_overwrite(
                    self.task,
                    addr,
                    buf.len() as mach_vm_size_t,
                    buf.as_mut_ptr() as mach_vm_address_t,
                    &mut size,
                );
                kr == KERN_SUCCESS && size == buf.len() as mach_vm_size_t
            }
        }
    }

    /// Find the 32MB GameCube RAM region in Dolphin's memory
    unsafe fn find_gc_ram_base(task: task_t) -> Option<mach_vm_address_t> {
        const GC_RAM_SIZE: mach_vm_size_t = 32 * 1024 * 1024; // 32MB

        let mut address: mach_vm_address_t = 0;
        let mut size: mach_vm_size_t = 0;
        let mut info: vm_region_basic_info_data_64_t = mem::zeroed();
        let mut info_count = mem::size_of::<vm_region_basic_info_data_64_t>() as u32
            / mem::size_of::<i32>() as u32;
        let mut object_name: mach_port_t = 0;

        loop {
            let kr = mach_vm_region(
                task,
                &mut address,
                &mut size,
                VM_REGION_BASIC_INFO_64,
                &mut info as *mut _ as *mut i32,
                &mut info_count,
                &mut object_name,
            );

            if kr != KERN_SUCCESS {
                break;
            }

            // Look for a region of exactly 32MB that is readable and writable
            // Dolphin's GC RAM is typically mapped as rw-
            if size == GC_RAM_SIZE {
                return Some(address);
            }

            address += size;
        }

        None
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
