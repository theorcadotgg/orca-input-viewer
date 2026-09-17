import { applyState, buildDiagram, computeViewerState, decodeConfig, makePortOptions } from './lib/orca-viewer.js';

// DOM Elements
const svg = document.getElementById('orcaDiagram');
const adapterChip = document.getElementById('adapterChip');
const adapterStatus = document.getElementById('adapterStatus');
const configChip = document.getElementById('configChip');
const configStatus = document.getElementById('configStatus');
const portSelect = document.getElementById('portSelect');
const profileSelect = document.getElementById('profileSelect');
const streamButton = document.getElementById('streamButton');
const connectionNotice = document.getElementById('connectionNotice');
const configNotice = document.getElementById('configNotice');
const outputsNotice = document.getElementById('outputsNotice');
const loadConfigBtn = document.getElementById('loadConfig');
const showOverlay = document.getElementById('showOverlay');
const hideOverlay = document.getElementById('hideOverlay');
const startObs = document.getElementById('startObs');
const stopObs = document.getElementById('stopObs');
const obsUrlRow = document.getElementById('obsUrlRow');
const obsUrl = document.getElementById('obsUrl');
const copyObs = document.getElementById('copyObs');
const frameRate = document.getElementById('frameRate');
const inputMode = document.getElementById('inputMode');
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
function setNotice(el, text, tone = 'muted') {
  if (!el) return;
  el.textContent = text || '';
  if (tone !== 'muted') el.dataset.tone = tone;
  else delete el.dataset.tone;
  el.classList.toggle('hidden', !text);
}

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

const ADAPTER_STATUS = {
  offline: { label: 'Offline', title: 'Adapter not connected - start a stream to connect' },
  waiting: { label: 'Waiting', title: 'Streaming - waiting for input reports' },
  connected: { label: 'Connected', title: 'Adapter detected - stream stopped' },
  live: { label: 'Live', title: 'Receiving input reports' },
};

function setAdapterStatus(connected, streaming = false) {
  const state = connected ? (streaming ? 'live' : 'connected') : streaming ? 'waiting' : 'offline';
  adapterChip.dataset.state = state;
  adapterChip.title = ADAPTER_STATUS[state].title;
  adapterStatus.textContent = ADAPTER_STATUS[state].label;
}

function setConfigStatus(loaded) {
  configChip.dataset.state = loaded ? 'loaded' : 'default';
  configChip.title = loaded ? 'Mappings loaded from device' : 'Using built-in default mappings';
  configStatus.textContent = loaded ? 'Device mappings' : 'Default mappings';
}

function setInputMode(mode, processName) {
  const label = mode === 'usb' ? 'USB adapter' : mode === 'dolphin' ? processName || '' : '';
  inputMode.classList.toggle('hidden', !label);
  inputMode.textContent = label;
  if (!label) {
    inputMode.title = '';
    return;
  }
  inputMode.dataset.state = mode === 'dolphin' ? 'dolphin' : 'usb';
  inputMode.title = mode === 'dolphin' ? `Reading from ${processName}` : 'Reading directly from USB adapter';
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
      frameRate.textContent = '-- fps';
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
  // Drop the "waiting for inputs" hint once any port reports data.
  if (connectionNotice.dataset.tone === 'info' && report?.ports?.some((p) => p.connected)) {
    setNotice(connectionNotice, '');
  }
  render();
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
  const dolphin = selectedInputMode === 'dolphin';
  modeDolphin.classList.toggle('active', dolphin);
  modeStandalone.classList.toggle('active', !dolphin);
  modeDolphin.setAttribute('aria-pressed', String(dolphin));
  modeStandalone.setAttribute('aria-pressed', String(!dolphin));
}

modeDolphin.addEventListener('click', () => {
  selectedInputMode = 'dolphin';
  updateModeToggle();
});

modeStandalone.addEventListener('click', () => {
  selectedInputMode = 'usb';
  updateModeToggle();
});

// The backend reads the input mode when the stream starts, so lock it while streaming.
function setModeLocked(locked) {
  [modeDolphin, modeStandalone].forEach((btn) => {
    btn.disabled = locked;
    btn.title = locked ? 'Stop the stream to change input mode' : '';
  });
}

function setStreamUI(streaming) {
  isStreaming = streaming;
  streamButton.classList.toggle('live', streaming);
  streamButton.classList.toggle('primary', !streaming);
  streamButton.textContent = streaming ? 'Stop Stream' : 'Start Stream';
  setModeLocked(streaming);
}

async function startStream() {
  streamButton.disabled = true;
  streamButton.classList.remove('primary');
  streamButton.textContent = 'Starting...';
  setAdapterStatus(false, true);

  try {
    await tauriInvoke('start_adapter_stream', { mode: selectedInputMode });
    setStreamUI(true);
    setNotice(
      connectionNotice,
      selectedInputMode === 'dolphin'
        ? 'Waiting for Dolphin/Slippi to report inputs.'
        : 'Waiting for adapter to report inputs.',
      'info'
    );
  } catch (err) {
    console.error('Failed to start stream:', err);
    setAdapterStatus(false, false);
    setNotice(connectionNotice, `Stream error: ${err.message || err}`, 'error');
    setStreamUI(false);
  } finally {
    streamButton.disabled = false;
  }
}

async function stopStream() {
  streamButton.disabled = true;
  try {
    await tauriInvoke('stop_adapter_stream');
    setStreamUI(false);
    setAdapterStatus(false, false);
    setInputMode(null);
    setNotice(connectionNotice, '');
    frameRate.textContent = '-- fps';
    frameRate.classList.remove('active');
  } catch (err) {
    console.error('Failed to stop stream:', err);
    setNotice(connectionNotice, `Stop error: ${err.message || err}`, 'error');
  } finally {
    streamButton.disabled = false;
  }
}

streamButton.addEventListener('click', () => void (isStreaming ? stopStream() : startStream()));

// macOS: reading Dolphin's RAM needs the emulator signed with get-task-allow.
enableMacAccess.addEventListener('click', async () => {
  enableMacAccess.disabled = true;
  setNotice(connectionNotice, 'Signing Dolphin for macOS memory access...', 'info');
  try {
    setNotice(connectionNotice, await tauriInvoke('enable_dolphin_debug_access'), 'success');
  } catch (err) {
    setNotice(connectionNotice, `${err.message || err}`, 'error');
  } finally {
    enableMacAccess.disabled = false;
  }
});

loadConfigBtn.addEventListener('click', async () => {
  loadConfigBtn.disabled = true;
  setNotice(configNotice, 'Waiting for Orca config mode...', 'info');

  try {
    const res = await tauriInvoke('load_config');
    const blobBase64 = res.blob_base64 || res.blobBase64;
    config = decodeConfig(blobBase64);
    setConfigStatus(true);
    setNotice(configNotice, 'Config loaded. Device rebooted to normal mode.', 'success');
    selectedProfile = config.activeProfile ?? 0;
    updateProfileOptions();
    render();
    // Sync profile to backend and notify overlay of config change
    tauriInvoke('set_selected_profile', { profile: selectedProfile }).catch(() => {});
    tauriEmit('config_changed', { blobBase64, profile: selectedProfile });
  } catch (err) {
    console.error('Config load failed:', err);
    setConfigStatus(false);
    setNotice(configNotice, `Failed: ${err.message || err}`, 'error');
  } finally {
    loadConfigBtn.disabled = false;
  }
});

showOverlay.addEventListener('click', async () => {
  try {
    await tauriInvoke('show_overlay_window');
    setNotice(outputsNotice, '');
  } catch (err) {
    console.error('Failed to show overlay:', err);
    setNotice(outputsNotice, `Could not show overlay: ${err.message || err}`, 'error');
  }
});

hideOverlay.addEventListener('click', async () => {
  try {
    await tauriInvoke('hide_overlay_window');
    setNotice(outputsNotice, '');
  } catch (err) {
    console.error('Failed to hide overlay:', err);
    setNotice(outputsNotice, `Could not hide overlay: ${err.message || err}`, 'error');
  }
});

startObs.addEventListener('click', async () => {
  startObs.disabled = true;
  try {
    const res = await tauriInvoke('start_overlay_server');
    obsUrl.value = res.url || '';
    obsUrlRow.classList.toggle('hidden', !obsUrl.value);
    setNotice(outputsNotice, '');
  } catch (err) {
    console.error('Failed to start OBS server:', err);
    setNotice(outputsNotice, `Could not start OBS server: ${err.message || err}`, 'error');
  } finally {
    startObs.disabled = false;
  }
});

stopObs.addEventListener('click', async () => {
  stopObs.disabled = true;
  try {
    await tauriInvoke('stop_overlay_server');
    obsUrl.value = '';
    obsUrlRow.classList.add('hidden');
    setNotice(outputsNotice, '');
  } catch (err) {
    console.error('Failed to stop OBS server:', err);
    setNotice(outputsNotice, `Could not stop OBS server: ${err.message || err}`, 'error');
  } finally {
    stopObs.disabled = false;
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
    setNotice(outputsNotice, 'Could not copy the URL.', 'error');
  }
});

// Bootstrap
async function bootstrap() {
  updatePortOptions();
  updateProfileOptions();
  updateModeToggle(); // Initialize mode toggle state
  render();
  // Dolphin's memory is only gated behind code signing on macOS.
  if (navigator.userAgent.includes('Mac')) {
    enableMacAccess.classList.remove('hidden');
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
    setStreamUI(false);
    setAdapterStatus(false, false);
    setInputMode(null);
    setNotice(connectionNotice, `Adapter error: ${event.payload}`, 'error');
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
