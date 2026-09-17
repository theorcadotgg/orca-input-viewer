import { applyState, buildDiagram, computeViewerState, decodeConfig, makePortOptions } from './lib/orca-viewer.js';

// DOM Elements
const svg = document.getElementById('orcaDiagram');
const adapterDot = document.getElementById('adapterDot');
const adapterStatus = document.getElementById('adapterStatus');
const configDot = document.getElementById('configDot');
const configStatus = document.getElementById('configStatus');
const portSelect = document.getElementById('portSelect');
const profileSelect = document.getElementById('profileSelect');
const startStream = document.getElementById('startStream');
const stopStream = document.getElementById('stopStream');
const loadConfigBtn = document.getElementById('loadConfig');
const loadHint = document.getElementById('loadHint');
const showOverlay = document.getElementById('showOverlay');
const hideOverlay = document.getElementById('hideOverlay');
const startObs = document.getElementById('startObs');
const stopObs = document.getElementById('stopObs');
const obsUrl = document.getElementById('obsUrl');
const copyObs = document.getElementById('copyObs');
const frameRate = document.getElementById('frameRate');
const inputMode = document.getElementById('inputMode');
const macAccessRow = document.getElementById('macAccessRow');
const enableMacAccess = document.getElementById('enableMacAccess');
const modeDolphin = document.getElementById('modeDolphin');
const modeStandalone = document.getElementById('modeStandalone');
const checkUpdatesBtn = document.getElementById('checkUpdates');
const appVersionEl = document.getElementById('appVersion');

// Update Modal Elements
const updateModal = document.getElementById('updateModal');
const updateBackdrop = document.getElementById('updateBackdrop');
const updateTitle = document.getElementById('updateTitle');
const updateSubtitle = document.getElementById('updateSubtitle');
const updateNotes = document.getElementById('updateNotes');
const updateLater = document.getElementById('updateLater');
const updateNow = document.getElementById('updateNow');
const updateStatus = document.getElementById('updateStatus');

// State
let config = decodeConfig(null);
let selectedPort = 0;
let selectedProfile = config.activeProfile ?? 0;
let lastReport = null;
let lastFrameTime = 0;
let frameCount = 0;
let isStreaming = false;
let selectedInputMode = 'dolphin'; // Default to Dolphin mode
let pendingUpdate = null;
let isAutoPortActive = false;

// Build the SVG diagram
buildDiagram(svg);

// Tauri IPC helpers with better error handling
function tauriInvoke(command, args = {}) {
  const tauri = window.__TAURI__;
  if (!tauri) {
    console.warn('Tauri not available - running in browser mode');
    return Promise.reject(new Error('Tauri not available'));
  }
  if (tauri.core?.invoke) return tauri.core.invoke(command, args);
  if (tauri.invoke) return tauri.invoke(command, args);
  return tauri.tauri?.invoke ? tauri.tauri.invoke(command, args) : Promise.reject(new Error('Invoke unavailable'));
}

function tauriListen(event, handler) {
  const tauri = window.__TAURI__;
  if (!tauri) return Promise.resolve(() => {});
  if (tauri.event?.listen) return tauri.event.listen(event, handler);
  return tauri.listen ? tauri.listen(event, handler) : Promise.resolve(() => {});
}

function tauriEmit(event, payload) {
  const tauri = window.__TAURI__;
  if (!tauri) return Promise.resolve();
  if (tauri.event?.emit) return tauri.event.emit(event, payload);
  if (tauri.emit) return tauri.emit(event, payload);
  return Promise.resolve();
}

function hasUpdater() {
  const tauri = window.__TAURI__;
  return Boolean(tauri?.updater?.check && tauri?.updater?.Update);
}

function openUpdateModal() {
  if (!updateModal) return;
  updateModal.classList.remove('hidden');
}

function closeUpdateModal() {
  if (!updateModal) return;
  updateModal.classList.add('hidden');
}

function setUpdatePrimaryAction(enabled, label) {
  if (!updateNow) return;
  updateNow.disabled = !enabled;
  if (label) updateNow.textContent = label;
}

function setUpdateStatus(text) {
  if (!updateStatus) return;
  updateStatus.textContent = text || '';
  updateStatus.style.color = '';
}

function setUpdateError(text) {
  if (!updateStatus) return;
  updateStatus.textContent = text || '';
  updateStatus.style.color = 'var(--danger)';
}

function normalizeReleaseNotes(body) {
  if (!body) return 'No release notes provided.';
  if (typeof body === 'string') return body.trim() || 'No release notes provided.';
  return String(body);
}

async function loadAppVersion() {
  if (!appVersionEl) return;
  try {
    const version = await tauriInvoke('get_app_version');
    if (version) appVersionEl.textContent = `v${version}`;
  } catch {
    // No-op; version is optional UI affordance.
  }
}

async function checkForUpdates({ userInitiated = false } = {}) {
  if (!window.__TAURI__) return;
  if (userInitiated) {
    if (updateTitle) updateTitle.textContent = 'Updates';
    if (updateSubtitle) updateSubtitle.textContent = '';
    if (updateNotes) updateNotes.textContent = '';
    setUpdatePrimaryAction(false, 'Update');
    setUpdateStatus('Checking for updates…');
    openUpdateModal();
  }
  if (!hasUpdater()) {
    if (userInitiated) setUpdateError('Updater is not available in this build.');
    return;
  }

  try {
    const update = await window.__TAURI__.updater.check();
    pendingUpdate = update;

    if (!update) {
      if (userInitiated) {
        setUpdateStatus('No updates available.');
      }
      return;
    }

    if (updateTitle) updateTitle.textContent = 'Update available';
    if (updateSubtitle) {
      updateSubtitle.textContent = `v${update.version} available (current: v${update.currentVersion})`;
    }
    if (updateNotes) {
      updateNotes.textContent = normalizeReleaseNotes(update.body);
    }
    setUpdatePrimaryAction(true, 'Update & Restart');
    setUpdateStatus('');
    openUpdateModal();
  } catch (err) {
    if (userInitiated) {
      setUpdateError(err?.message ?? String(err));
    } else {
      // Silent on background checks.
      console.debug('Update check failed:', err);
    }
  }
}

async function downloadAndInstallUpdate() {
  if (!pendingUpdate) return;
  if (updateNow) updateNow.disabled = true;
  if (updateLater) updateLater.disabled = true;
  setUpdateStatus('Downloading update…');

  let totalBytes = null;
  let downloadedBytes = 0;

  try {
    await pendingUpdate.downloadAndInstall((event) => {
      if (event?.event === 'started') {
        totalBytes = event?.data?.contentLength ?? null;
        downloadedBytes = 0;
        setUpdateStatus('Downloading update…');
      } else if (event?.event === 'progress') {
        downloadedBytes += event?.data?.chunkLength ?? 0;
        if (typeof totalBytes === 'number' && totalBytes > 0) {
          const pct = Math.min(100, Math.max(0, Math.round((downloadedBytes / totalBytes) * 100)));
          setUpdateStatus(`Downloading update… ${pct}%`);
        } else {
          setUpdateStatus('Downloading update…');
        }
      } else if (event?.event === 'finished') {
        setUpdateStatus('Launching installer…');
      }
    });
    // On Windows, the updater will launch the installer and exit the app.
  } catch (err) {
    setUpdateError(err?.message ?? String(err));
    if (updateNow) updateNow.disabled = false;
    if (updateLater) updateLater.disabled = false;
  }
}

// UI Update Functions
function updateProfileOptions() {
  profileSelect.innerHTML = '';
  config.profileLabels.forEach((label, idx) => {
    const opt = document.createElement('option');
    opt.value = String(idx);
    opt.textContent = label;
    profileSelect.appendChild(opt);
  });
  profileSelect.value = String(selectedProfile);
}

function updatePortOptions() {
  portSelect.innerHTML = '';
  makePortOptions(4).forEach((opt) => {
    const option = document.createElement('option');
    option.value = String(opt.value);
    option.textContent = opt.label;
    portSelect.appendChild(option);
  });
  portSelect.value = String(selectedPort);
  setPortSelectAutoState(isAutoPortActive);
}

function setAdapterStatus(connected, streaming = false) {
  if (connected) {
    adapterDot.classList.add('active');
    adapterStatus.textContent = streaming ? 'Live' : 'Connected';
  } else {
    adapterDot.classList.remove('active');
    adapterStatus.textContent = streaming ? 'Waiting...' : 'Offline';
  }
}

function setConfigStatus(loaded) {
  if (loaded) {
    configDot.classList.add('active');
    configStatus.textContent = 'Loaded';
  } else {
    configDot.classList.remove('active');
    configStatus.textContent = 'Default';
  }
}

function setInputMode(mode, processName) {
  inputMode.classList.remove('usb', 'dolphin');
  if (mode === 'dolphin') {
    inputMode.classList.add('dolphin');
    inputMode.textContent = processName ? `Dolphin` : 'Dolphin';
    inputMode.title = processName ? `Reading from ${processName}` : 'Reading from Dolphin memory';
  } else if (mode === 'usb') {
    inputMode.classList.add('usb');
    inputMode.textContent = 'USB';
    inputMode.title = 'Reading directly from USB adapter';
  } else {
    inputMode.textContent = '';
    inputMode.title = '';
  }
}

function resolveAutoPort(report) {
  const autoPort = report?.auto_port;
  if (Number.isInteger(autoPort) && autoPort >= 0 && autoPort <= 3) {
    return autoPort;
  }
  return null;
}

function setPortSelectAutoState(active) {
  portSelect.disabled = active;
  if (active) {
    portSelect.title = 'Auto-following Slippi local player port';
  } else {
    portSelect.title = '';
  }
}

function render() {
  const report = lastReport?.ports?.find((p) => p.port === selectedPort);
  const viewerState = computeViewerState(report, config, selectedProfile);
  applyState(svg, viewerState);

  const connected = report?.connected;
  setAdapterStatus(connected, isStreaming);

  const now = performance.now();
  frameCount += 1;
  if (now - lastFrameTime > 1000) {
    if (isStreaming && frameCount > 0) {
      frameRate.textContent = `${frameCount} fps`;
      frameRate.classList.add('active');
    } else {
      frameRate.textContent = 'Idle';
      frameRate.classList.remove('active');
    }
    frameCount = 0;
    lastFrameTime = now;
  }
}

function onInputReport(report) {
  lastReport = report;
  const autoPort = resolveAutoPort(report);
  isAutoPortActive = autoPort !== null;
  if (autoPort !== null) {
    selectedPort = autoPort;
    portSelect.value = String(selectedPort);
  }
  setPortSelectAutoState(isAutoPortActive);
  render();
}

// Collapsible Section Toggle
function initCollapsibleSections() {
  document.querySelectorAll('.collapsible .section-toggle').forEach((toggle) => {
    toggle.addEventListener('click', () => {
      const section = toggle.closest('.collapsible');
      section.classList.toggle('collapsed');
    });
  });
}

// Event Handlers
portSelect.addEventListener('change', (e) => {
  selectedPort = Number(e.target.value);
  render();
});

profileSelect.addEventListener('change', (e) => {
  selectedProfile = Number(e.target.value);
  render();
  // Sync profile to backend (for OBS overlay) and notify overlay window
  tauriInvoke('set_selected_profile', { profile: selectedProfile }).catch(() => {});
  tauriEmit('profile_changed', { profile: selectedProfile });
});

// Input Mode Toggle
function updateModeToggle() {
  modeDolphin.classList.toggle('active', selectedInputMode === 'dolphin');
  modeStandalone.classList.toggle('active', selectedInputMode === 'usb');
}

modeDolphin.addEventListener('click', () => {
  selectedInputMode = 'dolphin';
  updateModeToggle();
});

modeStandalone.addEventListener('click', () => {
  selectedInputMode = 'usb';
  updateModeToggle();
});

// macOS: reading Dolphin's RAM needs the emulator signed with get-task-allow.
enableMacAccess.addEventListener('click', async () => {
  enableMacAccess.disabled = true;
  loadHint.style.color = '';
  loadHint.textContent = 'Signing Dolphin for macOS memory access...';
  try {
    loadHint.textContent = await tauriInvoke('enable_dolphin_debug_access');
  } catch (err) {
    loadHint.style.color = 'var(--danger)';
    loadHint.textContent = `${err.message || err}`;
  } finally {
    enableMacAccess.disabled = false;
  }
});

startStream.addEventListener('click', async () => {
  startStream.disabled = true;
  setAdapterStatus(false, true);

  if (selectedInputMode === 'dolphin') {
    loadHint.textContent = 'Waiting for Dolphin/Slippi...';
    loadHint.style.color = '';
  }

  try {
    await tauriInvoke('start_adapter_stream', { mode: selectedInputMode });
    isStreaming = true;
    startStream.textContent = 'Streaming...';
  } catch (err) {
    console.error('Failed to start stream:', err);
    setAdapterStatus(false, false);
    // Show error in hint
    loadHint.textContent = `Stream error: ${err.message || err}`;
    loadHint.style.color = 'var(--danger)';
  } finally {
    startStream.disabled = false;
  }
});

stopStream.addEventListener('click', async () => {
  stopStream.disabled = true;
  try {
    await tauriInvoke('stop_adapter_stream');
    isStreaming = false;
    startStream.textContent = 'Start Stream';
    setAdapterStatus(false, false);
    setInputMode(null);
    frameRate.textContent = 'Idle';
    frameRate.classList.remove('active');
  } catch (err) {
    console.error('Failed to stop stream:', err);
  } finally {
    stopStream.disabled = false;
  }
});

loadConfigBtn.addEventListener('click', async () => {
  loadConfigBtn.disabled = true;
  loadHint.textContent = 'Waiting for Orca config mode...';
  loadHint.style.color = '';

  try {
    const res = await tauriInvoke('load_config');
    const blobBase64 = res.blob_base64 || res.blobBase64;
    config = decodeConfig(blobBase64);
    setConfigStatus(true);
    loadHint.textContent = 'Config loaded. Device rebooted to normal mode.';
    loadHint.style.color = 'var(--success)';
    selectedProfile = config.activeProfile ?? 0;
    updateProfileOptions();
    render();
    // Sync profile to backend and notify overlay of config change
    tauriInvoke('set_selected_profile', { profile: selectedProfile }).catch(() => {});
    tauriEmit('config_changed', { blobBase64, profile: selectedProfile });
  } catch (err) {
    console.error('Config load failed:', err);
    setConfigStatus(false);
    loadHint.textContent = `Failed: ${err.message || err}`;
    loadHint.style.color = 'var(--danger)';
  } finally {
    loadConfigBtn.disabled = false;
  }
});

showOverlay.addEventListener('click', async () => {
  try {
    await tauriInvoke('show_overlay_window');
  } catch (err) {
    console.error('Failed to show overlay:', err);
  }
});

hideOverlay.addEventListener('click', async () => {
  try {
    await tauriInvoke('hide_overlay_window');
  } catch (err) {
    console.error('Failed to hide overlay:', err);
  }
});

startObs.addEventListener('click', async () => {
  try {
    const res = await tauriInvoke('start_overlay_server');
    obsUrl.value = res.url || '';
  } catch (err) {
    console.error('Failed to start OBS server:', err);
  }
});

stopObs.addEventListener('click', async () => {
  try {
    await tauriInvoke('stop_overlay_server');
    obsUrl.value = '';
  } catch (err) {
    console.error('Failed to stop OBS server:', err);
  }
});

copyObs.addEventListener('click', async () => {
  if (!obsUrl.value) return;
  try {
    await navigator.clipboard.writeText(obsUrl.value);
    const originalHTML = copyObs.innerHTML;
    copyObs.innerHTML = '<svg width="14" height="14" viewBox="0 0 14 14"><path d="M3 7l3 3 5-6" stroke="var(--success)" stroke-width="2" fill="none"/></svg>';
    setTimeout(() => {
      copyObs.innerHTML = originalHTML;
    }, 1500);
  } catch (err) {
    console.error('Failed to copy:', err);
  }
});

// Bootstrap
async function bootstrap() {
  initCollapsibleSections();
  updatePortOptions();
  updateProfileOptions();
  updateModeToggle(); // Initialize mode toggle state
  render();
  // Dolphin's memory is only gated behind code signing on macOS.
  if (navigator.userAgent.includes('Mac')) {
    macAccessRow.classList.remove('hidden');
  }
  await loadAppVersion();

  if (checkUpdatesBtn) {
    checkUpdatesBtn.addEventListener('click', () => void checkForUpdates({ userInitiated: true }));
  }
  if (updateLater) updateLater.addEventListener('click', closeUpdateModal);
  if (updateBackdrop) updateBackdrop.addEventListener('click', closeUpdateModal);
  if (updateNow) updateNow.addEventListener('click', () => void downloadAndInstallUpdate());

  // Listen for input reports
  await tauriListen('input_report', (event) => {
    onInputReport(event.payload);
  });

  // Listen for adapter errors
  await tauriListen('adapter_error', (event) => {
    console.error('Adapter error:', event.payload);
    setAdapterStatus(false, false);
    setInputMode(null);
    isStreaming = false;
    startStream.textContent = 'Start Stream';
    loadHint.textContent = `Adapter error: ${event.payload}`;
    loadHint.style.color = 'var(--danger)';
  });

  // Listen for input mode changes
  await tauriListen('input_mode', (event) => {
    const { mode, process_name } = event.payload;
    setInputMode(mode, process_name);
  });

  // Try to load any cached config
  try {
    const blob = await tauriInvoke('get_config_blob');
    if (blob) {
      config = decodeConfig(blob);
      setConfigStatus(true);
      selectedProfile = config.activeProfile ?? 0;
      updateProfileOptions();
      render();
    }
  } catch (err) {
    // Config not available, use defaults
    console.debug('No cached config:', err);
  }

  // Check for updates in the background.
  void checkForUpdates({ userInitiated: false });
}

bootstrap();
