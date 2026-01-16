use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use base64::{engine::general_purpose, Engine as _};
use crc32c::crc32c;
use rusb::{Context, DeviceHandle, Error as UsbError, UsbContext};
use serde::Serialize;
use serialport::{SerialPortInfo, SerialPortType};
use tauri::{AppHandle, Emitter, Manager, State};
use tiny_http::{Header, ListenAddr, Response, Server};

const ADAPTER_VID: u16 = 0x057e;
const ADAPTER_PID: u16 = 0x0337;
const ADAPTER_INTERFACE: u8 = 0;
const ADAPTER_IN_EP: u8 = 0x81;
const ADAPTER_OUT_EP: u8 = 0x02;
const ADAPTER_CMD_BEGIN_POLLING: u8 = 0x13;
const ADAPTER_CMD_STOP_POLLING: u8 = 0x14;

const CONFIG_VID: u16 = 0x2e8a;
const CONFIG_PID: u16 = 0x000a;
const CONFIG_BAUD: u32 = 115200;

const ORCA_CONFIG_PROTO_MAGIC: u32 = 0x4143524f;
const ORCA_CONFIG_PROTO_VERSION: u8 = 1;
const ORCA_MSG_REQUEST: u8 = 1;
const ORCA_MSG_ERROR: u8 = 3;

const ORCA_CMD_GET_INFO: u8 = 1;
const ORCA_CMD_READ_BLOB: u8 = 4;
const ORCA_CMD_REBOOT: u8 = 11;

const ORCA_SETTINGS_BLOB_SIZE: u32 = 16384;

#[derive(Clone, Debug, Serialize)]
struct Buttons {
    a: bool,
    b: bool,
    x: bool,
    y: bool,
    z: bool,
    l: bool,
    r: bool,
    start: bool,
    dpad_up: bool,
    dpad_down: bool,
    dpad_left: bool,
    dpad_right: bool,
}

#[derive(Clone, Debug, Serialize)]
struct Axes {
    stick_x: f32,
    stick_y: f32,
    substick_x: f32,
    substick_y: f32,
    trigger_l: f32,
    trigger_r: f32,
}

#[derive(Clone, Debug, Serialize)]
struct PortReport {
    port: u8,
    connected: bool,
    buttons: Buttons,
    axes: Axes,
}

#[derive(Clone, Debug, Serialize)]
struct InputReport {
    ports: Vec<PortReport>,
}

#[derive(Default)]
struct SharedState {
    last_report: Mutex<Option<InputReport>>,
    config_blob_b64: Mutex<Option<String>>,
}

#[derive(Default)]
struct AppState {
    shared: Arc<SharedState>,
    adapter_running: Arc<AtomicBool>,
    adapter_handle: Mutex<Option<JoinHandle<()>>>,
    overlay_running: Arc<AtomicBool>,
    overlay_port: Mutex<Option<u16>>,
    overlay_handle: Mutex<Option<JoinHandle<()>>>,
}

#[derive(Debug)]
struct OrcaFrame {
    msg_type: u8,
    seq: u32,
    payload: Vec<u8>,
}

#[derive(Debug, Serialize)]
struct DeviceInfo {
    schema_id: u32,
    settings_major: u8,
    settings_minor: u8,
    blob_size: u32,
    max_chunk: u32,
    slot_count: u32,
}

#[derive(Debug, Serialize)]
struct LoadConfigResult {
    blob_base64: String,
    info: DeviceInfo,
}

#[derive(Debug, Serialize)]
struct OverlayServerInfo {
    url: String,
}

fn read_u32_le(buf: &[u8], offset: usize) -> u32 {
    let bytes = [buf[offset], buf[offset + 1], buf[offset + 2], buf[offset + 3]];
    u32::from_le_bytes(bytes)
}

fn encode_frame(msg_type: u8, seq: u32, payload: &[u8]) -> Vec<u8> {
    let mut header = [0u8; 16];
    header[0..4].copy_from_slice(&ORCA_CONFIG_PROTO_MAGIC.to_le_bytes());
    header[4] = ORCA_CONFIG_PROTO_VERSION;
    header[5] = msg_type;
    header[6..8].copy_from_slice(&(payload.len() as u16).to_le_bytes());
    header[8..12].copy_from_slice(&seq.to_le_bytes());
    header[12..16].copy_from_slice(&0u32.to_le_bytes());

    let mut combined = Vec::with_capacity(header.len() + payload.len());
    combined.extend_from_slice(&header);
    combined.extend_from_slice(payload);
    let crc = crc32c(&combined);
    header[12..16].copy_from_slice(&crc.to_le_bytes());

    let mut out = Vec::with_capacity(header.len() + payload.len());
    out.extend_from_slice(&header);
    out.extend_from_slice(payload);
    out
}

fn try_decode_frame(buffer: &mut Vec<u8>) -> Result<Option<OrcaFrame>, String> {
    if buffer.len() < 16 {
        return Ok(None);
    }

    let magic = read_u32_le(buffer, 0);
    if magic != ORCA_CONFIG_PROTO_MAGIC {
        return Err("Bad magic".to_string());
    }
    let proto_ver = buffer[4];
    if proto_ver != ORCA_CONFIG_PROTO_VERSION {
        return Err("Bad protocol version".to_string());
    }

    let msg_type = buffer[5];
    let payload_len = u16::from_le_bytes([buffer[6], buffer[7]]) as usize;
    let seq = read_u32_le(buffer, 8);
    let crc = read_u32_le(buffer, 12);

    let total_len = 16 + payload_len;
    if buffer.len() < total_len {
        return Ok(None);
    }

    let mut header = buffer[..16].to_vec();
    header[12..16].copy_from_slice(&0u32.to_le_bytes());
    let payload = buffer[16..total_len].to_vec();
    let mut combined = Vec::with_capacity(header.len() + payload.len());
    combined.extend_from_slice(&header);
    combined.extend_from_slice(&payload);
    let expected = crc32c(&combined);
    if crc != expected {
        return Err("Bad CRC32C".to_string());
    }

    let remaining = buffer.split_off(total_len);
    *buffer = remaining;

    Ok(Some(OrcaFrame {
        msg_type,
        seq,
        payload,
    }))
}

fn read_frame(port: &mut dyn serialport::SerialPort, timeout: Duration) -> Result<OrcaFrame, String> {
    let start = Instant::now();
    let mut rx: Vec<u8> = Vec::new();
    let mut buf = [0u8; 512];

    loop {
        if start.elapsed() > timeout {
            return Err("Timeout waiting for response".to_string());
        }

        match port.read(&mut buf) {
            Ok(n) if n > 0 => {
                rx.extend_from_slice(&buf[..n]);
                if let Some(frame) = try_decode_frame(&mut rx)? {
                    return Ok(frame);
                }
            }
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {}
            Err(e) => return Err(format!("Serial read error: {e}")),
        }
    }
}

fn send_and_read(port: &mut dyn serialport::SerialPort, seq: &mut u32, payload: &[u8]) -> Result<OrcaFrame, String> {
    let frame = encode_frame(ORCA_MSG_REQUEST, *seq, payload);
    *seq = seq.wrapping_add(1);
    port.write_all(&frame).map_err(|e| format!("Serial write error: {e}"))?;
    read_frame(port, Duration::from_millis(1500))
}

fn read_device_info(frame: &OrcaFrame) -> Result<DeviceInfo, String> {
    if frame.payload.len() < 16 {
        return Err("Bad GET_INFO response".to_string());
    }
    let payload = &frame.payload;
    let schema_id = read_u32_le(payload, 4);
    let blob_size = read_u32_le(payload, 8).max(ORCA_SETTINGS_BLOB_SIZE);
    let max_chunk = read_u32_le(payload, 12).max(256);
    let slot_count = if payload.len() >= 20 {
        read_u32_le(payload, 16).max(1)
    } else {
        1
    };

    Ok(DeviceInfo {
        schema_id,
        settings_major: payload[1],
        settings_minor: payload[2],
        blob_size,
        max_chunk,
        slot_count,
    })
}

fn read_blob(port: &mut dyn serialport::SerialPort, seq: &mut u32, blob_size: u32, max_chunk: u32) -> Result<Vec<u8>, String> {
    let mut blob = vec![0u8; blob_size as usize];
    let mut offset = 0u32;
    while offset < blob_size {
        let len = std::cmp::min(max_chunk, blob_size - offset);
        let mut payload = [0u8; 12];
        payload[0] = ORCA_CMD_READ_BLOB;
        payload[4..8].copy_from_slice(&offset.to_le_bytes());
        payload[8..12].copy_from_slice(&len.to_le_bytes());

        let frame = send_and_read(port, seq, &payload)?;
        if frame.msg_type == ORCA_MSG_ERROR {
            let cmd = frame.payload.get(0).copied().unwrap_or(0);
            let err = frame.payload.get(1).copied().unwrap_or(0);
            return Err(format!("Device error cmd={cmd} err={err}"));
        }
        if frame.payload.len() < 12 {
            return Err("Bad READ_BLOB response".to_string());
        }
        let got_offset = read_u32_le(&frame.payload, 4);
        let got_len = read_u32_le(&frame.payload, 8) as usize;
        if got_offset != offset || got_len as u32 != len {
            return Err("READ_BLOB mismatch".to_string());
        }
        let data = frame.payload[12..12 + got_len].to_vec();
        blob[offset as usize..offset as usize + got_len].copy_from_slice(&data);
        offset += len;
    }
    Ok(blob)
}

fn config_timeout_error() -> String {
    #[cfg(target_os = "windows")]
    {
        "Timed out waiting for Orca config mode. Ensure the device is in config mode and appears as a COM port. You may need to install RP2040 USB drivers.".to_string()
    }
    #[cfg(not(target_os = "windows"))]
    {
        "Timed out waiting for Orca config mode. Press and hold the config button while connecting to enter config mode.".to_string()
    }
}

fn wait_for_config_port(timeout: Duration) -> Result<SerialPortInfo, String> {
    let start = Instant::now();

    loop {
        if start.elapsed() > timeout {
            return Err(config_timeout_error());
        }

        let ports = match serialport::available_ports() {
            Ok(p) => p,
            Err(_) => {
                // On some systems, port enumeration can temporarily fail
                thread::sleep(Duration::from_millis(250));
                continue;
            }
        };

        // Look for RP2040 in BOOTSEL/config mode
        if let Some(found) = ports.into_iter().find(|p| matches_config_port(p)) {
            // Give the port a moment to stabilize after detection
            thread::sleep(Duration::from_millis(100));
            return Ok(found);
        }

        thread::sleep(Duration::from_millis(250));
    }
}

fn matches_config_port(info: &SerialPortInfo) -> bool {
    match &info.port_type {
        SerialPortType::UsbPort(usb) => usb.vid == CONFIG_VID && usb.pid == CONFIG_PID,
        _ => false,
    }
}

fn adapter_not_found_error() -> String {
    #[cfg(target_os = "windows")]
    {
        "GameCube adapter not found. On Windows, ensure WinUSB driver is installed using Zadig (https://zadig.akeo.ie)".to_string()
    }
    #[cfg(target_os = "macos")]
    {
        "GameCube adapter not found. Ensure the adapter is connected and no other app is using it.".to_string()
    }
    #[cfg(target_os = "linux")]
    {
        "GameCube adapter not found. You may need to add udev rules for the device.".to_string()
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        "GameCube adapter not found.".to_string()
    }
}

fn adapter_open_error() -> String {
    #[cfg(target_os = "windows")]
    {
        "Could not open adapter. Install WinUSB driver using Zadig.".to_string()
    }
    #[cfg(not(target_os = "windows"))]
    {
        "Could not open adapter. Check permissions.".to_string()
    }
}

fn adapter_claim_error(e: impl std::fmt::Display) -> String {
    #[cfg(target_os = "windows")]
    {
        format!("USB claim failed: {e}. Ensure WinUSB driver is installed via Zadig.")
    }
    #[cfg(not(target_os = "windows"))]
    {
        format!("USB claim failed: {e}. Another app may be using the adapter.")
    }
}

fn open_adapter() -> Result<(Context, DeviceHandle<Context>), String> {
    let context = Context::new().map_err(|e| format!("USB init failed: {e}"))?;

    // Check if the device exists first
    let devices = context.devices().map_err(|e| format!("USB device enumeration failed: {e}"))?;
    let mut found_device = false;
    for device in devices.iter() {
        if let Ok(desc) = device.device_descriptor() {
            if desc.vendor_id() == ADAPTER_VID && desc.product_id() == ADAPTER_PID {
                found_device = true;
                break;
            }
        }
    }

    if !found_device {
        return Err(adapter_not_found_error());
    }

    let handle = context
        .open_device_with_vid_pid(ADAPTER_VID, ADAPTER_PID)
        .ok_or_else(adapter_open_error)?;

    #[cfg(not(target_os = "windows"))]
    let _ = handle.set_auto_detach_kernel_driver(true);

    if let Err(e) = handle.claim_interface(ADAPTER_INTERFACE) {
        return Err(adapter_claim_error(e));
    }

    Ok((context, handle))
}

fn begin_polling(handle: &mut DeviceHandle<Context>) -> Result<(), UsbError> {
    let _ = handle.write_interrupt(
        ADAPTER_OUT_EP,
        &[ADAPTER_CMD_BEGIN_POLLING],
        Duration::from_millis(50),
    );
    Ok(())
}

fn stop_polling(handle: &mut DeviceHandle<Context>) {
    let _ = handle.write_interrupt(
        ADAPTER_OUT_EP,
        &[ADAPTER_CMD_STOP_POLLING],
        Duration::from_millis(50),
    );
}

fn parse_adapter_report(data: &[u8]) -> Option<InputReport> {
    let payload = if data.first().copied() == Some(0x21) && data.len() > 1 {
        &data[1..]
    } else {
        data
    };

    let payload = if payload.len() == 37 { &payload[1..] } else { payload };
    if payload.len() < 36 {
        return None;
    }

    let mut ports = Vec::with_capacity(4);
    for port in 0..4u8 {
        let base = (port as usize) * 9;
        let state = payload[base];
        let connected = (state & 0x10) != 0;
        let b1 = payload[base + 1];
        let b2 = payload[base + 2];

        let buttons = Buttons {
            a: (b1 & 0x01) != 0,
            b: (b1 & 0x02) != 0,
            x: (b1 & 0x04) != 0,
            y: (b1 & 0x08) != 0,
            start: (b1 & 0x10) != 0,
            dpad_left: (b2 & 0x01) != 0,
            dpad_right: (b2 & 0x02) != 0,
            dpad_down: (b2 & 0x04) != 0,
            dpad_up: (b2 & 0x08) != 0,
            z: (b2 & 0x10) != 0,
            r: (b2 & 0x20) != 0,
            l: (b2 & 0x40) != 0,
        };

        let stick_x = payload[base + 3];
        let stick_y = payload[base + 4];
        let sub_x = payload[base + 5];
        let sub_y = payload[base + 6];
        let trig_l = payload[base + 7];
        let trig_r = payload[base + 8];

        let axes = Axes {
            stick_x: (stick_x as f32 - 128.0) / 127.0,
            stick_y: (stick_y as f32 - 128.0) / 127.0,
            substick_x: (sub_x as f32 - 128.0) / 127.0,
            substick_y: (sub_y as f32 - 128.0) / 127.0,
            trigger_l: trig_l as f32 / 255.0,
            trigger_r: trig_r as f32 / 255.0,
        };

        ports.push(PortReport {
            port,
            connected,
            buttons,
            axes,
        });
    }

    Some(InputReport { ports })
}

#[tauri::command]
fn start_adapter_stream(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    if state.adapter_running.swap(true, Ordering::SeqCst) {
        return Ok(());
    }

    let running = state.adapter_running.clone();
    let shared = state.shared.clone();

    let handle = thread::spawn(move || {
        let (_ctx, mut usb) = match open_adapter() {
            Ok(ok) => ok,
            Err(err) => {
                let _ = app.emit("adapter_error", err);
                running.store(false, Ordering::SeqCst);
                return;
            }
        };

        let _ = begin_polling(&mut usb);
        let mut buf = [0u8; 64];

        while running.load(Ordering::SeqCst) {
            match usb.read_interrupt(ADAPTER_IN_EP, &mut buf, Duration::from_millis(100)) {
                Ok(len) => {
                    if let Some(report) = parse_adapter_report(&buf[..len]) {
                        if let Ok(mut guard) = shared.last_report.lock() {
                            *guard = Some(report.clone());
                        }
                        let _ = app.emit("input_report", report);
                    }
                }
                Err(UsbError::Timeout) => continue,
                Err(e) => {
                    let _ = app.emit("adapter_error", format!("Adapter read error: {e}"));
                    break;
                }
            }
        }

        stop_polling(&mut usb);
        running.store(false, Ordering::SeqCst);
    });

    *state.adapter_handle.lock().unwrap() = Some(handle);
    Ok(())
}

#[tauri::command]
fn stop_adapter_stream(state: State<'_, AppState>) -> Result<(), String> {
    state.adapter_running.store(false, Ordering::SeqCst);
    if let Some(handle) = state.adapter_handle.lock().unwrap().take() {
        let _ = handle.join();
    }
    Ok(())
}

#[tauri::command]
fn load_config(state: State<'_, AppState>) -> Result<LoadConfigResult, String> {
    let port_info = wait_for_config_port(Duration::from_secs(12))?;
    let mut port = serialport::new(&port_info.port_name, CONFIG_BAUD)
        .timeout(Duration::from_millis(120))
        .open()
        .map_err(|e| format!("Open config port failed: {e}"))?;

    let mut seq = 1u32;
    let frame = send_and_read(&mut *port, &mut seq, &[ORCA_CMD_GET_INFO])?;
    if frame.msg_type == ORCA_MSG_ERROR {
        let cmd = frame.payload.get(0).copied().unwrap_or(0);
        let err = frame.payload.get(1).copied().unwrap_or(0);
        return Err(format!("Device error cmd={cmd} err={err}"));
    }
    let info = read_device_info(&frame)?;
    let blob = read_blob(&mut *port, &mut seq, info.blob_size, info.max_chunk)?;

    let _ = send_and_read(&mut *port, &mut seq, &[ORCA_CMD_REBOOT]);

    let blob_base64 = general_purpose::STANDARD.encode(&blob);
    if let Ok(mut guard) = state.shared.config_blob_b64.lock() {
        *guard = Some(blob_base64.clone());
    }

    Ok(LoadConfigResult { blob_base64, info })
}

#[tauri::command]
fn set_config_blob(state: State<'_, AppState>, blob_base64: String) -> Result<(), String> {
    if let Ok(mut guard) = state.shared.config_blob_b64.lock() {
        *guard = Some(blob_base64);
    }
    Ok(())
}

#[tauri::command]
fn get_config_blob(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let guard = state
        .shared
        .config_blob_b64
        .lock()
        .map_err(|_| "Config lock failed".to_string())?;
    Ok(guard.clone())
}

#[tauri::command]
fn show_overlay_window(app: AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("overlay")
        .ok_or_else(|| "Overlay window not found".to_string())?;
    window.show().map_err(map_tauri_err)?;
    window.set_always_on_top(true).map_err(map_tauri_err)?;
    Ok(())
}

#[tauri::command]
fn hide_overlay_window(app: AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("overlay")
        .ok_or_else(|| "Overlay window not found".to_string())?;
    window.hide().map_err(map_tauri_err)?;
    Ok(())
}

#[tauri::command]
fn start_overlay_server(state: State<'_, AppState>) -> Result<OverlayServerInfo, String> {
    if state.overlay_running.swap(true, Ordering::SeqCst) {
        let port = state.overlay_port.lock().unwrap().unwrap_or(0);
        return Ok(OverlayServerInfo {
            url: format!("http://127.0.0.1:{port}/overlay"),
        });
    }

    let running = state.overlay_running.clone();
    let shared = state.shared.clone();
    let server = match Server::http("127.0.0.1:0") {
        Ok(server) => server,
        Err(e) => {
            state.overlay_running.store(false, Ordering::SeqCst);
            return Err(format!("Server start failed: {e}"));
        }
    };
    let addr = server.server_addr();
    let port = match addr {
        ListenAddr::IP(ip) => ip.port(),
        ListenAddr::Unix(_) => 0,
    };
    *state.overlay_port.lock().unwrap() = Some(port);

    let handle = thread::spawn(move || {
        while running.load(Ordering::SeqCst) {
            match server.recv_timeout(Duration::from_millis(200)) {
                Ok(Some(request)) => {
                    let url = request.url().to_string();
                    let method = request.method().as_str();
                    if method != "GET" {
                        let _ = request.respond(Response::empty(405));
                        continue;
                    }

                    let response = match url.as_str() {
                        "/" | "/overlay" | "/overlay.html" => Response::from_string(OVERLAY_HTML)
                            .with_header(content_type("text/html; charset=utf-8")),
                        "/overlay.js" => Response::from_string(OVERLAY_JS)
                            .with_header(content_type("application/javascript; charset=utf-8")),
                        "/styles.css" => Response::from_string(STYLES_CSS)
                            .with_header(content_type("text/css; charset=utf-8")),
                        "/lib/orca-viewer.js" => Response::from_string(ORCA_VIEWER_JS)
                            .with_header(content_type("application/javascript; charset=utf-8")),
                        "/assets/ORCATOPBLANKTEMPLATE-Edge_Cuts.svg" => Response::from_data(ORCA_SVG)
                            .with_header(content_type("image/svg+xml")),
                        "/config" => {
                            let blob = shared.config_blob_b64.lock().ok().and_then(|g| g.clone());
                            let body = serde_json::json!({ "blobBase64": blob });
                            Response::from_string(body.to_string())
                                .with_header(content_type("application/json"))
                        }
                        "/state" => {
                            let report = shared.last_report.lock().ok().and_then(|g| g.clone());
                            let blob = shared.config_blob_b64.lock().ok().and_then(|g| g.clone());
                            let body = serde_json::json!({ "input": report, "blobBase64": blob });
                            Response::from_string(body.to_string())
                                .with_header(content_type("application/json"))
                        }
                        _ => Response::from_string("Not Found").with_status_code(404),
                    };

                    let _ = request.respond(response.with_header(cache_control()));
                }
                Ok(None) => continue,
                Err(_) => continue,
            }
        }
    });

    *state.overlay_handle.lock().unwrap() = Some(handle);
    Ok(OverlayServerInfo {
        url: format!("http://127.0.0.1:{port}/overlay"),
    })
}

#[tauri::command]
fn stop_overlay_server(state: State<'_, AppState>) -> Result<(), String> {
    state.overlay_running.store(false, Ordering::SeqCst);
    if let Some(handle) = state.overlay_handle.lock().unwrap().take() {
        let _ = handle.join();
    }
    Ok(())
}

fn content_type(value: &str) -> Header {
    Header::from_bytes(&b"Content-Type"[..], value.as_bytes()).unwrap()
}

fn cache_control() -> Header {
    Header::from_bytes(&b"Cache-Control"[..], &b"no-store"[..]).unwrap()
}

static OVERLAY_HTML: &str = include_str!("../../ui/overlay.html");
static OVERLAY_JS: &str = include_str!("../../ui/overlay.js");
static STYLES_CSS: &str = include_str!("../../ui/styles.css");
static ORCA_VIEWER_JS: &str = include_str!("../../ui/lib/orca-viewer.js");
static ORCA_SVG: &[u8] = include_bytes!("../../ui/assets/ORCATOPBLANKTEMPLATE-Edge_Cuts.svg");

fn map_tauri_err(err: tauri::Error) -> String {
    err.to_string()
}

fn main() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            start_adapter_stream,
            stop_adapter_stream,
            load_config,
            set_config_blob,
            get_config_blob,
            show_overlay_window,
            hide_overlay_window,
            start_overlay_server,
            stop_overlay_server,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
