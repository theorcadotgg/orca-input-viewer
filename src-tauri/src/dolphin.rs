//! Dolphin memory reading for controller input
//!
//! Reads controller state directly from Dolphin's emulated GameCube RAM,
//! allowing the input viewer to work while Dolphin has the USB adapter claimed.

use crate::{Axes, Buttons, InputReport, PortReport};
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

// ---------------------------------------------------------------------------
// Which port is the local player on?
// ---------------------------------------------------------------------------
//
// Slippi online does not leave the local player on a fixed port, so guessing
// (or defaulting to port 0) shows the opponent's inputs whenever the match
// assigned the local player to another port. Slippi's own game-side code keeps
// the answer in emulated RAM; the addresses below are Melee NTSC 1.02, matching
// project-slippi/slippi-ssbm-asm (`Online/Online.s`, `Online/Core/InitOnlinePlay.asm`).

/// Scene ID. Slippi's `getMinorMajor` macro reads the u32 at 0x80479D30 and
/// keeps the low 16 bits as `(minor << 8) | major`.
const SCENE_MAJOR_ADDR: u32 = 0x8047_9D30;
const SCENE_MINOR_ADDR: u32 = 0x8047_9D33;

const SCENE_ONLINE: u8 = 0x08;
const SCENE_ONLINE_CSS: u8 = 0x00;
const SCENE_ONLINE_SSS: u8 = 0x01;
const SCENE_ONLINE_IN_GAME: u8 = 0x02;
// Minor 0x03 (results), 0x04 (splash) and 0x05 (game setup) carry no usable
// local port, so those scenes leave the manual port selection alone.

/// Merged into the scene table, this is `-0x5108(r13)` with r13 = 0x804DB6A0:
/// the port the local player is using during the online menus, which Slippi
/// reads to find the local player's cursor on the CSS.
const ONLINE_MENU_LOCAL_PORT_ADDR: u32 = 0x804D_6598;

/// `-0x49E4(r13)`: pointer to the online data buffer, allocated when an online
/// match starts and preserved across rollback savestates. Its first two bytes
/// are the ports the local player and the opponent were assigned.
const ONLINE_DATA_BUF_PTR_ADDR: u32 = 0x804D_6CBC;
const ONLINE_DATA_BUF_LOCAL_PORT: u32 = 0x00;
const ONLINE_DATA_BUF_OPPONENT_PORT: u32 = 0x01;

/// Emulated RAM is the first 24MB of the GameCube address space, and Slippi's
/// buffers are heap allocated inside it.
fn gc_ram_ptr(ptr: u32) -> Option<u32> {
    (0x8000_0000..0x8180_0000).contains(&ptr).then_some(ptr)
}

/// Port the local player's controller is plugged into, or `None` when the
/// current scene says nothing about it (offline play, replays, menus without a
/// match) - the caller then falls back to the port the user picked.
fn detect_slippi_local_port<M: EmuRam>(ram: &M) -> Option<u8> {
    if ram.read_u8_emu(SCENE_MAJOR_ADDR)? != SCENE_ONLINE {
        return None;
    }

    match ram.read_u8_emu(SCENE_MINOR_ADDR)? {
        SCENE_ONLINE_CSS | SCENE_ONLINE_SSS => ram
            .read_u8_emu(ONLINE_MENU_LOCAL_PORT_ADDR)
            .filter(|port| *port <= 3),
        SCENE_ONLINE_IN_GAME => {
            let buf = gc_ram_ptr(ram.read_u32_be_emu(ONLINE_DATA_BUF_PTR_ADDR)?)?;
            let local = ram.read_u8_emu(buf + ONLINE_DATA_BUF_LOCAL_PORT)?;
            let opponent = ram.read_u8_emu(buf + ONLINE_DATA_BUF_OPPONENT_PORT)?;
            // A match always has two players on two different ports; anything
            // else means the buffer is not this game's and is not trustworthy.
            (local <= 3 && opponent <= 3 && local != opponent).then_some(local)
        }
        _ => None,
    }
}

/// Byte access to the emulated RAM Dolphin has mapped into its own address
/// space. Reads are all-or-nothing, and each platform reader implements the
/// raw read; everything above them is shared.
trait EmuRam {
    fn read_emu(&self, emu_addr: u32, buf: &mut [u8]) -> bool;

    fn read_u8_emu(&self, emu_addr: u32) -> Option<u8> {
        let mut buf = [0u8; 1];
        self.read_emu(emu_addr, &mut buf).then_some(buf[0])
    }

    fn read_u32_be_emu(&self, emu_addr: u32) -> Option<u32> {
        let mut buf = [0u8; 4];
        self.read_emu(emu_addr, &mut buf)
            .then_some(u32::from_be_bytes(buf))
    }
}

// ---------------------------------------------------------------------------
// Controller state in emulated RAM
// ---------------------------------------------------------------------------
//
// Melee keeps its four controllers in one array. The struct starting at
// 0x804C1FAC is Melee's own controller data rather than the raw PADStatus
// buffer, which is what carries the analog trigger bytes and stick floats the
// viewer draws. Everything below runs the same on every platform; the platform
// readers only supply raw reads of emulated RAM.

const MELEE_CONTROLLER_BASE: u32 = 0x804C_1FAC;
const CONTROLLER_STRUCT_SIZE: u32 = 0x44; // 68 bytes per controller

/// Offsets within the Melee controller struct
const OFFSET_BUTTONS: usize = 0x00; // u32 buttons pressed
const OFFSET_TRIGGER_L: usize = 0x1C; // u8 L trigger analog
const OFFSET_TRIGGER_R: usize = 0x1D; // u8 R trigger analog
const OFFSET_STICK_X: usize = 0x20; // f32 main stick X
const OFFSET_STICK_Y: usize = 0x24; // f32 main stick Y
const OFFSET_CSTICK_X: usize = 0x28; // f32 C-stick X
const OFFSET_CSTICK_Y: usize = 0x2C; // f32 C-stick Y
const OFFSET_PLUGGED: usize = 0x41; // u8 plugged status

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

fn read_f32_be(buf: &[u8], offset: usize) -> f32 {
    f32::from_be_bytes([buf[offset], buf[offset + 1], buf[offset + 2], buf[offset + 3]])
}

/// One port's controller struct, or a disconnected placeholder when the struct
/// cannot be read - a single unreadable port must not blank the whole viewer.
fn read_port<M: EmuRam>(ram: &M, port: u8) -> PortReport {
    let controller_addr = MELEE_CONTROLLER_BASE + (port as u32 * CONTROLLER_STRUCT_SIZE);

    let mut buf = [0u8; CONTROLLER_STRUCT_SIZE as usize];
    if !ram.read_emu(controller_addr, &mut buf) {
        return PortReport {
            port,
            connected: false,
            buttons: default_buttons(),
            axes: default_axes(),
        };
    }

    let buttons_raw = u32::from_be_bytes([
        buf[OFFSET_BUTTONS],
        buf[OFFSET_BUTTONS + 1],
        buf[OFFSET_BUTTONS + 2],
        buf[OFFSET_BUTTONS + 3],
    ]);

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
        stick_x: read_f32_be(&buf, OFFSET_STICK_X),
        stick_y: read_f32_be(&buf, OFFSET_STICK_Y),
        substick_x: read_f32_be(&buf, OFFSET_CSTICK_X),
        substick_y: read_f32_be(&buf, OFFSET_CSTICK_Y),
        trigger_l: buf[OFFSET_TRIGGER_L] as f32 / 255.0,
        trigger_r: buf[OFFSET_TRIGGER_R] as f32 / 255.0,
    };

    PortReport {
        port,
        // Port 1 stays visible even with nothing plugged in; the others only
        // light up once Melee reports them as plugged.
        connected: port == 0 || buf[OFFSET_PLUGGED] != 0,
        buttons,
        axes,
    }
}

/// Controller state as the UI consumes it, including Slippi's local port.
fn read_state<M: EmuRam>(ram: &M) -> InputReport {
    InputReport {
        ports: (0..4u8).map(|port| read_port(ram, port)).collect(),
        auto_port: detect_slippi_local_port(ram),
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use super::*;
    use winapi::shared::minwindef::{DWORD, FALSE, LPCVOID};
    use winapi::um::handleapi::CloseHandle;
    use winapi::um::memoryapi::{ReadProcessMemory, VirtualQueryEx};
    use winapi::um::processthreadsapi::OpenProcess;
    use winapi::um::winnt::{
        HANDLE, MEMORY_BASIC_INFORMATION, MEM_MAPPED, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
    };

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
            Ok(read_state(self))
        }

        fn emu_to_host_addr(&self, emu_addr: u32) -> usize {
            self.gc_ram_base + (emu_addr & 0x01FF_FFFF) as usize
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

    impl EmuRam for DolphinReader {
        fn read_emu(&self, emu_addr: u32, buf: &mut [u8]) -> bool {
            self.read_memory(self.emu_to_host_addr(emu_addr), buf)
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
    use mach2::kern_return::KERN_SUCCESS;
    use mach2::mach_types::task_t;
    use mach2::port::mach_port_t;
    use mach2::traps::task_for_pid;
    use mach2::vm::{mach_vm_read_overwrite, mach_vm_region};
    use mach2::vm_region::{vm_region_basic_info_data_64_t, VM_REGION_BASIC_INFO_64};
    use mach2::vm_types::{mach_vm_address_t, mach_vm_size_t};
    use std::mem;

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
            Ok(read_state(self))
        }

        fn emu_to_host_addr(&self, emu_addr: u32) -> mach_vm_address_t {
            self.gc_ram_base + (emu_addr & 0x01FF_FFFF) as u64
        }

        fn read_memory(&self, addr: mach_vm_address_t, buf: &mut [u8]) -> bool {
            unsafe { read_raw(self.task, addr, buf) }
        }
    }

    impl EmuRam for DolphinReader {
        fn read_emu(&self, emu_addr: u32, buf: &mut [u8]) -> bool {
            self.read_memory(self.emu_to_host_addr(emu_addr), buf)
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
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use std::os::unix::fs::FileExt;

    /// A readable mapping from /proc/<pid>/maps.
    #[derive(Debug, Clone, Copy)]
    pub(super) struct Region {
        pub(super) start: usize,
        pub(super) end: usize,
    }

    impl Region {
        pub(super) fn size(&self) -> usize {
            self.end - self.start
        }
    }

    pub struct DolphinReader {
        mem: File,
        gc_ram_base: usize,
    }

    impl DolphinReader {
        pub fn new(process: &DolphinProcess) -> Result<Self, String> {
            // Reading another process's memory is a ptrace operation, so the
            // kernel gates /proc/<pid>/mem on the same permission check.
            let mem = File::open(format!("/proc/{}/mem", process.pid)).map_err(|e| {
                format!(
                    "Linux blocked access to Dolphin's memory (pid {}, {e}). Run \
                     `sudo sysctl -w kernel.yama.ptrace_scope=0`, then Start Stream again.",
                    process.pid
                )
            })?;

            let regions = read_maps(process.pid)?;
            match find_gc_ram_base(&mem, &regions) {
                Some(base) => Ok(DolphinReader { mem, gc_ram_base: base }),
                None => Err("Could not find GameCube RAM in Dolphin memory. \
                             Make sure a game is running in Dolphin."
                    .to_string()),
            }
        }

        pub fn read_controller_state(&self) -> Result<InputReport, String> {
            Ok(read_state(self))
        }

        fn emu_to_host_addr(&self, emu_addr: u32) -> usize {
            self.gc_ram_base + (emu_addr & 0x01FF_FFFF) as usize
        }

        fn read_memory(&self, addr: usize, buf: &mut [u8]) -> bool {
            // pread on /proc/<pid>/mem: reads of unmapped pages fail with EIO,
            // which is how a not-yet-loaded game surfaces here.
            self.mem
                .read_at(buf, addr as u64)
                .is_ok_and(|bytes| bytes == buf.len())
        }
    }

    impl EmuRam for DolphinReader {
        fn read_emu(&self, emu_addr: u32, buf: &mut [u8]) -> bool {
            self.read_memory(self.emu_to_host_addr(emu_addr), buf)
        }
    }

    fn read_maps(pid: u32) -> Result<Vec<Region>, String> {
        let file = File::open(format!("/proc/{pid}/maps"))
            .map_err(|e| format!("Could not read Dolphin's memory map: {e}"))?;

        Ok(parse_maps(BufReader::new(file)))
    }

    /// Readable regions from a /proc/<pid>/maps stream. Malformed and
    /// unreadable lines are skipped rather than failing the whole read: the map
    /// contains entries (/dev/shm, stacks, reserved ranges) the reader cannot
    /// touch anyway.
    pub(super) fn parse_maps(reader: impl BufRead) -> Vec<Region> {
        let mut regions = Vec::new();
        for line in reader.lines().map_while(Result::ok) {
            let mut fields = line.split_whitespace();
            let (Some(range), Some(perms)) = (fields.next(), fields.next()) else {
                continue;
            };
            if !perms.starts_with('r') {
                continue;
            }
            let Some((start, end)) = range.split_once('-') else {
                continue;
            };
            if let (Ok(start), Ok(end)) = (
                usize::from_str_radix(start, 16),
                usize::from_str_radix(end, 16),
            ) {
                regions.push(Region { start, end });
            }
        }

        regions
    }

    /// Every GameCube disc header is copied to emulated 0x80000000 at boot, so the
    /// start of the RAM mapping reads back as the 8-character disc ID ("GALE01"
    /// plus the maker code for Melee). Dolphin also maps large anonymous regions
    /// for MEM2 and its caches, so the header - not the size alone - identifies
    /// the RAM mapping.
    fn has_disc_header(mem: &File, base: usize) -> bool {
        let mut id = [0u8; 8];
        mem.read_at(&mut id, base as u64).is_ok_and(|n| n == id.len())
            && id
                .iter()
                .all(|byte| byte.is_ascii_digit() || matches!(byte, b'A'..=b'Z'))
    }

    /// Find MEM1's host mapping. Dolphin maps MEM1 as a 32MB anonymous region on
    /// 64-bit Linux, and the region carries the disc header once a game is
    /// loaded. Before a game is loaded no region has a header, so fall back to
    /// the first 32MB region.
    fn find_gc_ram_base(mem: &File, regions: &[Region]) -> Option<usize> {
        const GC_RAM_SIZE: usize = 32 * 1024 * 1024; // MEM1 (24MB) plus the 8MB tail
        const MEM1_SIZE: usize = 24 * 1024 * 1024;

        let mut first_size_match: Option<usize> = None;

        for region in regions {
            // Smaller than MEM1 itself cannot hold emulated RAM at offset 0.
            if region.size() < MEM1_SIZE {
                continue;
            }
            if has_disc_header(mem, region.start) {
                return Some(region.start);
            }
            if region.size() == GC_RAM_SIZE && first_size_match.is_none() {
                first_size_match = Some(region.start);
            }
        }

        first_size_match
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

/// The Linux reader is exercised against a real process: the test binary
/// re-executes itself as a helper that maps a stand-in for Dolphin's RAM, so
/// maps parsing, disc-header discovery, /proc/<pid>/mem reads, address
/// translation and controller parsing all run against the kernel rather than a
/// mock.
#[cfg(all(test, target_os = "linux"))]
mod linux_tests {
    use super::linux::{parse_maps, DolphinReader};
    use super::*;
    use std::io::{BufRead, BufReader, Cursor, Write};
    use std::process::{Child, Command, Stdio};

    const HELPER_ENV: &str = "ORCA_RAM_HELPER";
    const HELPER_TEST: &str = "dolphin::linux_tests::plants_emulated_ram_for_the_parent";

    /// Emulated address -> offset inside the RAM mapping.
    fn ram_offset(emu_addr: u32) -> usize {
        (emu_addr & 0x01FF_FFFF) as usize
    }

    #[test]
    fn parses_readable_regions_from_proc_maps() {
        let maps = "\
5560a000-5560b000 r--p 00000000 08:02 1048669 /usr/bin/dolphin-emu
7f3c00000000-7f3c02000000 rw-p 00000000 00:00 0
7f3c02000000-7f3c02100000 ---p 00000000 00:00 0
garbage
7f3c10000000-7f3c10400000 r-xp 00000000 08:02 78 /usr/lib/libfoo.so
";
        let regions = parse_maps(Cursor::new(maps));

        assert_eq!(regions.len(), 3, "readable regions: {regions:?}");
        assert_eq!(regions[1].start, 0x7f3c00000000);
        assert_eq!(regions[1].size(), 32 * 1024 * 1024);
    }

    #[test]
    fn plants_emulated_ram_for_the_parent() {
        if std::env::var_os(HELPER_ENV).is_none() {
            return; // Regular run: the parent's test does the asserting.
        }

        const SIZE: usize = 32 * 1024 * 1024;
        let ptr = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                SIZE,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
                -1,
                0,
            )
        };
        assert_ne!(ptr, libc::MAP_FAILED, "helper could not map emulated RAM");
        let ram = unsafe { std::slice::from_raw_parts_mut(ptr as *mut u8, SIZE) };

        // Dolphin copies the disc header to emulated 0x80000000 at boot, so the
        // RAM mapping starts with the disc ID.
        ram[..8].copy_from_slice(b"GALE0101");

        // Melee's controller array, with a controller in port 3.
        const PORT: usize = 2;
        let base = ram_offset(MELEE_CONTROLLER_BASE) + PORT * CONTROLLER_STRUCT_SIZE as usize;
        let controller = &mut ram[base..base + CONTROLLER_STRUCT_SIZE as usize];
        controller[OFFSET_BUTTONS..OFFSET_BUTTONS + 4]
            .copy_from_slice(&(PAD_BUTTON_A | PAD_BUTTON_START).to_be_bytes());
        controller[OFFSET_TRIGGER_L] = 128;
        controller[OFFSET_STICK_X..OFFSET_STICK_X + 4].copy_from_slice(&0.5f32.to_be_bytes());
        controller[OFFSET_PLUGGED] = 1;

        // Slippi's online scene, which names the local player's port.
        ram[ram_offset(SCENE_MAJOR_ADDR)] = SCENE_ONLINE;
        ram[ram_offset(SCENE_MINOR_ADDR)] = SCENE_ONLINE_IN_GAME;
        let ptr_addr = ram_offset(ONLINE_DATA_BUF_PTR_ADDR);
        ram[ptr_addr..ptr_addr + 4].copy_from_slice(&0x8010_0000u32.to_be_bytes());
        let buf = ram_offset(0x8010_0000);
        ram[buf + ONLINE_DATA_BUF_LOCAL_PORT as usize] = PORT as u8;
        ram[buf + ONLINE_DATA_BUF_OPPONENT_PORT as usize] = 1;

        println!("base=0x{:x}", ptr as usize);
        std::io::stdout().flush().expect("flush helper address");

        // Stay mapped until the parent closes stdin.
        let mut line = String::new();
        let _ = std::io::stdin().read_line(&mut line);
    }

    #[test]
    fn reads_controller_state_out_of_a_live_process() {
        if std::env::var_os(HELPER_ENV).is_some() {
            return;
        }

        let mut child = Command::new(std::env::current_exe().expect("test binary path"))
            .args(["--exact", HELPER_TEST, "--nocapture"])
            .env(HELPER_ENV, "1")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("spawn helper");
        let mut helper_output = wait_for_helper(&mut child);

        let process = DolphinProcess {
            pid: child.id(),
            name: "Slippi_Dolphin-x86_64.AppImage".to_string(),
        };
        let reader = DolphinReader::new(&process).expect("attach to the helper");
        let report = reader.read_controller_state().expect("controller state");

        let port = &report.ports[2];
        assert!(port.connected, "plugged port reads as connected");
        assert!(port.buttons.a && port.buttons.start, "{:?}", port.buttons);
        assert!(!port.buttons.b && !port.buttons.z, "{:?}", port.buttons);
        assert!((port.axes.stick_x - 0.5).abs() < 1e-6, "stick x {:?}", port.axes.stick_x);
        assert!((port.axes.trigger_l - 128.0 / 255.0).abs() < 1e-6);
        assert!(!report.ports[1].connected, "empty port reads as disconnected");
        assert_eq!(report.auto_port, Some(2), "Slippi's local port");

        // Closing the helper's stdin lets it exit; drain its output first, or
        // the harness prints a broken-pipe error when it writes its summary.
        drop(child.stdin.take());
        for line in helper_output.by_ref() {
            if line.is_err() {
                break;
            }
        }
        let _ = child.wait();
    }

    /// Block until the helper reports the mapping it planted, so a helper that
    /// dies on startup fails with its own output instead of hanging the suite.
    /// The returned reader must outlive the helper.
    fn wait_for_helper(child: &mut Child) -> std::io::Lines<BufReader<std::process::ChildStdout>> {
        let stdout = child.stdout.take().expect("helper stdout");
        let mut lines = BufReader::new(stdout).lines();
        for _ in 0..50 {
            match lines.next() {
                Some(Ok(line)) if line.starts_with("base=") => return lines,
                Some(Ok(_)) => continue,
                Some(Err(err)) => panic!("helper output error: {err}"),
                None => break,
            }
        }
        let status = child.wait();
        panic!("helper never reported its mapping (exit: {status:?})");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Sparse stand-in for emulated RAM. Unmapped bytes fail to read, the same
    /// way a real read of an unmapped address does.
    #[derive(Default)]
    struct FakeRam(HashMap<u32, u8>);

    impl FakeRam {
        fn byte(mut self, addr: u32, value: u8) -> Self {
            self.0.insert(addr, value);
            self
        }

        fn word(self, addr: u32, value: u32) -> Self {
            self.byte(addr, (value >> 24) as u8)
                .byte(addr + 1, (value >> 16) as u8)
                .byte(addr + 2, (value >> 8) as u8)
                .byte(addr + 3, value as u8)
        }

        fn scene(self, major: u8, minor: u8) -> Self {
            self.byte(SCENE_MAJOR_ADDR, major)
                .byte(SCENE_MINOR_ADDR, minor)
        }

        /// An online match: scene 0x08/0x02 with the online data buffer Slippi
        /// allocates when the match starts.
        fn online_match(self, buffer: u32, local: u8, opponent: u8) -> Self {
            self.scene(SCENE_ONLINE, SCENE_ONLINE_IN_GAME)
                .word(ONLINE_DATA_BUF_PTR_ADDR, buffer)
                .byte(buffer + ONLINE_DATA_BUF_LOCAL_PORT, local)
                .byte(buffer + ONLINE_DATA_BUF_OPPONENT_PORT, opponent)
        }
    }

    impl EmuRam for FakeRam {
        fn read_emu(&self, emu_addr: u32, buf: &mut [u8]) -> bool {
            for (offset, byte) in buf.iter_mut().enumerate() {
                match self.0.get(&(emu_addr.wrapping_add(offset as u32))) {
                    Some(value) => *byte = *value,
                    None => return false,
                }
            }
            true
        }
    }

    /// This is the bug: with the local player on port 2, the viewer used to
    /// follow the stale CSS match-state pointer and read the low byte of an
    /// allocation address as if it were a port (here 0, the opponent's port).
    #[test]
    fn online_match_follows_the_local_port() {
        let ram = FakeRam::default()
            .online_match(0x80B0_0000, 2, 0)
            .word(0x8000_5614, 0x80B0_0000)
            .byte(0x80B0_0003, 0);
        assert_eq!(detect_slippi_local_port(&ram), Some(2));
    }

    #[test]
    fn online_css_follows_the_port_in_use() {
        let ram = FakeRam::default()
            .scene(SCENE_ONLINE, SCENE_ONLINE_CSS)
            .byte(ONLINE_MENU_LOCAL_PORT_ADDR, 1);
        assert_eq!(detect_slippi_local_port(&ram), Some(1));
    }

    #[test]
    fn untrustworthy_buffers_are_not_a_port() {
        // Buffer slot never filled in (Dolphin started, no match this boot).
        let unset = FakeRam::default().scene(SCENE_ONLINE, SCENE_ONLINE_IN_GAME);
        assert_eq!(detect_slippi_local_port(&unset), None);

        // Slot holds something that is not a GameCube RAM pointer.
        let not_a_pointer = FakeRam::default()
            .scene(SCENE_ONLINE, SCENE_ONLINE_IN_GAME)
            .word(ONLINE_DATA_BUF_PTR_ADDR, 0x0000_0000);
        assert_eq!(detect_slippi_local_port(&not_a_pointer), None);

        // Buffer claims both players are on the same port, so it cannot say
        // which of the two is the local player.
        let same_port = FakeRam::default().online_match(0x80B0_0000, 1, 1);
        assert_eq!(detect_slippi_local_port(&same_port), None);
    }

    #[test]
    fn offline_and_replay_scenes_have_no_local_port() {
        let offline_versus = FakeRam::default().scene(0x02, 0x02);
        assert_eq!(detect_slippi_local_port(&offline_versus), None);

        let replay = FakeRam::default().scene(0x0E, 0x01);
        assert_eq!(detect_slippi_local_port(&replay), None);
    }
}
