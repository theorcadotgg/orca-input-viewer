//! Dolphin memory reading for controller input
//!
//! Reads controller state directly from Dolphin's emulated GameCube RAM,
//! allowing the input viewer to work while Dolphin has the USB adapter claimed.

use crate::InputReport;
use sysinfo::{ProcessRefreshKind, RefreshKind, System, UpdateKind};

/// Reduce a process name to lowercase letters and digits, so that the same
/// executable matches on every platform ("Dolphin.exe", "dolphin-emu",
/// "Slippi_Dolphin" and "Slippi Dolphin" all normalise usefully).
fn normalize_process_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

/// Dolphin and its forks (Dolphin, Slippi, Ishiiruka) on all platforms.
pub(crate) fn is_dolphin_process_name(name: &str) -> bool {
    let normalized = normalize_process_name(name);
    normalized.starts_with("dolphin") || normalized.starts_with("slippidolphin")
}

#[derive(Debug)]
pub struct DolphinProcess {
    pub pid: u32,
    pub name: String,
}

/// Find a running Dolphin or Slippi process
pub fn find_dolphin_process() -> Option<DolphinProcess> {
    let system = System::new_with_specifics(
        RefreshKind::new()
            .with_processes(ProcessRefreshKind::new().with_exe(UpdateKind::Always)),
    );

    for (pid, process) in system.processes() {
        let name = process.name().to_string();
        // Some platforms report only the executable name, others part of a path;
        // the exe stem is the reliable field for macOS Slippi builds.
        let exe_name = process
            .exe()
            .and_then(|path| path.file_name())
            .map(|file| file.to_string_lossy().into_owned());

        if is_dolphin_process_name(&name)
            || exe_name.as_deref().is_some_and(is_dolphin_process_name)
        {
            return Some(DolphinProcess {
                pid: pid.as_u32(),
                name,
            });
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

    /// Memory addresses for Melee controller data
    /// NTSC 1.02: 0x804C1FAC
    /// This is Melee's controller struct, not raw PADStatus
    const MELEE_CONTROLLER_BASE: u32 = 0x804C1FAC;
    const CONTROLLER_STRUCT_SIZE: u32 = 0x44; // 68 bytes per controller
    const SCENE_MAJOR_ADDR: u32 = 0x80479D30;
    const SCENE_MINOR_ADDR: u32 = 0x80479D33;
    const MENU_PLAYER_ONE_PORT_ADDR: u32 = 0x804D6598;
    const CSSDT_BUF_ADDR: u32 = 0x80005614;
    const SLIPPI_LOCAL_INDEX_OFFSET: u32 = 0x03;

    const SCENE_VS_ONLINE: u8 = 0x08;
    const SCENE_VS_ONLINE_CSS: u8 = 0x00;
    const SCENE_VS_ONLINE_SSS: u8 = 0x01;
    const SCENE_VS_ONLINE_INGAME: u8 = 0x02;
    const SCENE_VS_ONLINE_VERSUS: u8 = 0x04;
    const SCENE_VS_ONLINE_RANKED: u8 = 0x05;

    /// Offsets within the Melee controller struct
    const OFFSET_BUTTONS: usize = 0x00;      // u32 buttons pressed
    const OFFSET_TRIGGER_L: usize = 0x1C;    // u8 L trigger analog
    const OFFSET_TRIGGER_R: usize = 0x1D;    // u8 R trigger analog
    const OFFSET_STICK_X: usize = 0x20;      // f32 main stick X
    const OFFSET_STICK_Y: usize = 0x24;      // f32 main stick Y
    const OFFSET_CSTICK_X: usize = 0x28;     // f32 C-stick X
    const OFFSET_CSTICK_Y: usize = 0x2C;     // f32 C-stick Y
    const OFFSET_PLUGGED: usize = 0x41;      // u8 plugged status

    /// GameCube button bit flags (u32, but only lower 16 bits used)
    const PAD_BUTTON_LEFT: u32 = 0x0001;
    const PAD_BUTTON_RIGHT: u32 = 0x0002;
    const PAD_BUTTON_DOWN: u32 = 0x0004;
    const PAD_BUTTON_UP: u32 = 0x0008;
    const PAD_BUTTON_Z: u32 = 0x0010;
    const PAD_BUTTON_R: u32 = 0x0020;
    const PAD_BUTTON_L: u32 = 0x0040;
    const PAD_BUTTON_A: u32 = 0x0100;
    const PAD_BUTTON_B: u32 = 0x0200;
    const PAD_BUTTON_X: u32 = 0x0400;
    const PAD_BUTTON_Y: u32 = 0x0800;
    const PAD_BUTTON_START: u32 = 0x1000;

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
                let host_addr = self.emu_to_host_addr(controller_addr);

                // Read the full controller struct (68 bytes)
                let mut buf = [0u8; 0x44];
                if !self.read_memory(host_addr, &mut buf) {
                    ports.push(PortReport {
                        port,
                        connected: false,
                        buttons: default_buttons(),
                        axes: default_axes(),
                    });
                    continue;
                }

                // Parse button state (offset 0x00, big-endian u32)
                let buttons_raw = u32::from_be_bytes([
                    buf[OFFSET_BUTTONS],
                    buf[OFFSET_BUTTONS + 1],
                    buf[OFFSET_BUTTONS + 2],
                    buf[OFFSET_BUTTONS + 3],
                ]);

                // Parse trigger values (bytes at specific offsets)
                let trigger_l = buf[OFFSET_TRIGGER_L];
                let trigger_r = buf[OFFSET_TRIGGER_R];

                // Parse stick values as big-endian floats
                let stick_x = f32::from_be_bytes([
                    buf[OFFSET_STICK_X],
                    buf[OFFSET_STICK_X + 1],
                    buf[OFFSET_STICK_X + 2],
                    buf[OFFSET_STICK_X + 3],
                ]);
                let stick_y = f32::from_be_bytes([
                    buf[OFFSET_STICK_Y],
                    buf[OFFSET_STICK_Y + 1],
                    buf[OFFSET_STICK_Y + 2],
                    buf[OFFSET_STICK_Y + 3],
                ]);
                let substick_x = f32::from_be_bytes([
                    buf[OFFSET_CSTICK_X],
                    buf[OFFSET_CSTICK_X + 1],
                    buf[OFFSET_CSTICK_X + 2],
                    buf[OFFSET_CSTICK_X + 3],
                ]);
                let substick_y = f32::from_be_bytes([
                    buf[OFFSET_CSTICK_Y],
                    buf[OFFSET_CSTICK_Y + 1],
                    buf[OFFSET_CSTICK_Y + 2],
                    buf[OFFSET_CSTICK_Y + 3],
                ]);

                // Check plugged status
                let plugged = buf[OFFSET_PLUGGED];

                // Port is connected if plugged byte is non-zero, or for port 0 always show connected
                let connected = port == 0 || plugged != 0;

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

            Ok(InputReport {
                ports,
                auto_port: self.detect_slippi_auto_port(),
            })
        }

        fn detect_slippi_auto_port(&self) -> Option<u8> {
            let scene_major = self.read_u8_emu(SCENE_MAJOR_ADDR)?;
            if scene_major != SCENE_VS_ONLINE {
                return None;
            }

            let scene_minor = self.read_u8_emu(SCENE_MINOR_ADDR)?;

            match scene_minor {
                SCENE_VS_ONLINE_INGAME | SCENE_VS_ONLINE_VERSUS | SCENE_VS_ONLINE_RANKED => {
                    let slippi_ptr = self.read_u32_emu_be(CSSDT_BUF_ADDR)?;
                    if (slippi_ptr & 0x8000_0000) == 0 {
                        return None;
                    }

                    let local_port = self.read_u8_emu(slippi_ptr.wrapping_add(SLIPPI_LOCAL_INDEX_OFFSET))?;
                    (local_port <= 3).then_some(local_port)
                }
                SCENE_VS_ONLINE_CSS | SCENE_VS_ONLINE_SSS => {
                    let menu_port = self.read_u8_emu(MENU_PLAYER_ONE_PORT_ADDR)?;
                    (menu_port <= 3).then_some(menu_port)
                }
                _ => None,
            }
        }

        fn emu_to_host_addr(&self, emu_addr: u32) -> usize {
            self.gc_ram_base + (emu_addr & 0x01FF_FFFF) as usize
        }

        fn read_u8_emu(&self, emu_addr: u32) -> Option<u8> {
            let host_addr = self.emu_to_host_addr(emu_addr);
            let mut buf = [0u8; 1];
            if self.read_memory(host_addr, &mut buf) {
                Some(buf[0])
            } else {
                None
            }
        }

        fn read_u32_emu_be(&self, emu_addr: u32) -> Option<u32> {
            let host_addr = self.emu_to_host_addr(emu_addr);
            let mut buf = [0u8; 4];
            if self.read_memory(host_addr, &mut buf) {
                Some(u32::from_be_bytes(buf))
            } else {
                None
            }
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

    /// Memory addresses for Melee controller data
    /// NTSC 1.02: 0x804C1FAC
    /// This is Melee's controller struct, not raw PADStatus
    const MELEE_CONTROLLER_BASE: u32 = 0x804C1FAC;
    const CONTROLLER_STRUCT_SIZE: u32 = 0x44; // 68 bytes per controller
    const SCENE_MAJOR_ADDR: u32 = 0x80479D30;
    const SCENE_MINOR_ADDR: u32 = 0x80479D33;
    const MENU_PLAYER_ONE_PORT_ADDR: u32 = 0x804D6598;
    const CSSDT_BUF_ADDR: u32 = 0x80005614;
    const SLIPPI_LOCAL_INDEX_OFFSET: u32 = 0x03;

    const SCENE_VS_ONLINE: u8 = 0x08;
    const SCENE_VS_ONLINE_CSS: u8 = 0x00;
    const SCENE_VS_ONLINE_SSS: u8 = 0x01;
    const SCENE_VS_ONLINE_INGAME: u8 = 0x02;
    const SCENE_VS_ONLINE_VERSUS: u8 = 0x04;
    const SCENE_VS_ONLINE_RANKED: u8 = 0x05;

    /// Offsets within the Melee controller struct
    const OFFSET_BUTTONS: usize = 0x00;      // u32 buttons pressed
    const OFFSET_TRIGGER_L: usize = 0x1C;    // u8 L trigger analog
    const OFFSET_TRIGGER_R: usize = 0x1D;    // u8 R trigger analog
    const OFFSET_STICK_X: usize = 0x20;      // f32 main stick X
    const OFFSET_STICK_Y: usize = 0x24;      // f32 main stick Y
    const OFFSET_CSTICK_X: usize = 0x28;     // f32 C-stick X
    const OFFSET_CSTICK_Y: usize = 0x2C;     // f32 C-stick Y
    const OFFSET_PLUGGED: usize = 0x41;      // u8 plugged status

    /// GameCube button bit flags (u32, but only lower 16 bits used)
    const PAD_BUTTON_LEFT: u32 = 0x0001;
    const PAD_BUTTON_RIGHT: u32 = 0x0002;
    const PAD_BUTTON_DOWN: u32 = 0x0004;
    const PAD_BUTTON_UP: u32 = 0x0008;
    const PAD_BUTTON_Z: u32 = 0x0010;
    const PAD_BUTTON_R: u32 = 0x0020;
    const PAD_BUTTON_L: u32 = 0x0040;
    const PAD_BUTTON_A: u32 = 0x0100;
    const PAD_BUTTON_B: u32 = 0x0200;
    const PAD_BUTTON_X: u32 = 0x0400;
    const PAD_BUTTON_Y: u32 = 0x0800;
    const PAD_BUTTON_START: u32 = 0x1000;

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
                        "macOS denied memory access to Dolphin (pid {}, error {}). The viewer needs \
                         the com.apple.security.cs.debugger entitlement and the emulator needs \
                         com.apple.security.get-task-allow. Run \
                         OrcaInputViewer/scripts/macos-enable-dolphin-access.sh once, then restart Dolphin.",
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
                let controller_addr = MELEE_CONTROLLER_BASE + (port as u32 * CONTROLLER_STRUCT_SIZE);
                let host_addr = self.emu_to_host_addr(controller_addr);

                // Read the full controller struct (68 bytes)
                let mut buf = [0u8; 0x44];
                if !self.read_memory(host_addr, &mut buf) {
                    ports.push(PortReport {
                        port,
                        connected: false,
                        buttons: default_buttons(),
                        axes: default_axes(),
                    });
                    continue;
                }

                // Parse button state (offset 0x00, big-endian u32)
                let buttons_raw = u32::from_be_bytes([
                    buf[OFFSET_BUTTONS],
                    buf[OFFSET_BUTTONS + 1],
                    buf[OFFSET_BUTTONS + 2],
                    buf[OFFSET_BUTTONS + 3],
                ]);

                // Parse trigger values (bytes at specific offsets)
                let trigger_l = buf[OFFSET_TRIGGER_L];
                let trigger_r = buf[OFFSET_TRIGGER_R];

                // Parse stick values as big-endian floats
                let stick_x = f32::from_be_bytes([
                    buf[OFFSET_STICK_X],
                    buf[OFFSET_STICK_X + 1],
                    buf[OFFSET_STICK_X + 2],
                    buf[OFFSET_STICK_X + 3],
                ]);
                let stick_y = f32::from_be_bytes([
                    buf[OFFSET_STICK_Y],
                    buf[OFFSET_STICK_Y + 1],
                    buf[OFFSET_STICK_Y + 2],
                    buf[OFFSET_STICK_Y + 3],
                ]);
                let substick_x = f32::from_be_bytes([
                    buf[OFFSET_CSTICK_X],
                    buf[OFFSET_CSTICK_X + 1],
                    buf[OFFSET_CSTICK_X + 2],
                    buf[OFFSET_CSTICK_X + 3],
                ]);
                let substick_y = f32::from_be_bytes([
                    buf[OFFSET_CSTICK_Y],
                    buf[OFFSET_CSTICK_Y + 1],
                    buf[OFFSET_CSTICK_Y + 2],
                    buf[OFFSET_CSTICK_Y + 3],
                ]);

                // Check plugged status
                let plugged = buf[OFFSET_PLUGGED];

                // Port is connected if plugged byte is non-zero, or for port 0 always show connected
                let connected = port == 0 || plugged != 0;

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

            Ok(InputReport {
                ports,
                auto_port: self.detect_slippi_auto_port(),
            })
        }

        fn detect_slippi_auto_port(&self) -> Option<u8> {
            let scene_major = self.read_u8_emu(SCENE_MAJOR_ADDR)?;
            if scene_major != SCENE_VS_ONLINE {
                return None;
            }

            let scene_minor = self.read_u8_emu(SCENE_MINOR_ADDR)?;

            match scene_minor {
                SCENE_VS_ONLINE_INGAME | SCENE_VS_ONLINE_VERSUS | SCENE_VS_ONLINE_RANKED => {
                    let slippi_ptr = self.read_u32_emu_be(CSSDT_BUF_ADDR)?;
                    if (slippi_ptr & 0x8000_0000) == 0 {
                        return None;
                    }

                    let local_port = self.read_u8_emu(slippi_ptr.wrapping_add(SLIPPI_LOCAL_INDEX_OFFSET))?;
                    (local_port <= 3).then_some(local_port)
                }
                SCENE_VS_ONLINE_CSS | SCENE_VS_ONLINE_SSS => {
                    let menu_port = self.read_u8_emu(MENU_PLAYER_ONE_PORT_ADDR)?;
                    (menu_port <= 3).then_some(menu_port)
                }
                _ => None,
            }
        }

        fn emu_to_host_addr(&self, emu_addr: u32) -> mach_vm_address_t {
            self.gc_ram_base + (emu_addr & 0x01FF_FFFF) as u64
        }

        fn read_u8_emu(&self, emu_addr: u32) -> Option<u8> {
            let host_addr = self.emu_to_host_addr(emu_addr);
            let mut buf = [0u8; 1];
            if self.read_memory(host_addr, &mut buf) {
                Some(buf[0])
            } else {
                None
            }
        }

        fn read_u32_emu_be(&self, emu_addr: u32) -> Option<u32> {
            let host_addr = self.emu_to_host_addr(emu_addr);
            let mut buf = [0u8; 4];
            if self.read_memory(host_addr, &mut buf) {
                Some(u32::from_be_bytes(buf))
            } else {
                None
            }
        }

        fn read_memory(&self, addr: mach_vm_address_t, buf: &mut [u8]) -> bool {
            unsafe { read_raw(self.task, addr, buf) }
        }
    }

    /// Read `buf.len()` bytes of another process's memory, all-or-nothing.
    unsafe fn read_raw(task: task_t, addr: mach_vm_address_t, buf: &mut [u8]) -> bool {
        let mut size: mach_vm_size_t = 0;
        let kr = mach_vm_read_overwrite(
            task,
            addr,
            buf.len() as mach_vm_size_t,
            buf.as_mut_ptr() as mach_vm_address_t,
            &mut size,
        );
        kr == KERN_SUCCESS && size == buf.len() as mach_vm_size_t
    }

    /// Every GameCube disc header is copied to emulated 0x80000000 at boot, so the
    /// start of the RAM mapping always reads back as the 8-character disc ID
    /// ("GALE01" + maker code for Melee). macOS maps several unrelated 32MB
    /// regions, so the header - not the size alone - identifies the real mapping.
    unsafe fn has_disc_header(task: task_t, base: mach_vm_address_t) -> bool {
        let mut id = [0u8; 8];
        read_raw(task, base, &mut id)
            && id
                .iter()
                .all(|byte| byte.is_ascii_digit() || matches!(byte, b'A'..=b'Z'))
    }

    /// Find the 32MB GameCube RAM mapping (MEM1 plus Dolphin's spare MEM2 window).
    ///
    /// A running game leaves its disc header at the start of the mapping, which is
    /// what separates it from the other 32MB regions macOS also maps. Before a game
    /// is loaded no region carries a header, so fall back to the first 32MB region.
    unsafe fn find_gc_ram_base(task: task_t) -> Option<mach_vm_address_t> {
        const GC_RAM_SIZE: mach_vm_size_t = 32 * 1024 * 1024; // 32MB

        let mut first_size_match: Option<mach_vm_address_t> = None;
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

            if size == GC_RAM_SIZE {
                if has_disc_header(task, address) {
                    return Some(address);
                }
                if first_size_match.is_none() {
                    first_size_match = Some(address);
                }
            }

            address += size;
        }

        first_size_match
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
