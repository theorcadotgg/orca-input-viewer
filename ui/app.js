import { overlayTemplate } from './lib/overlayTemplate.js';
import { buildDefaultConfig, decodeBase64ToBytes, tryParseSettingsBlob } from './lib/settingsBlob.js';
import { computeMapping, createOverlay, updateOverlay } from './lib/overlayRenderer.js';

const tauri = window.__TAURI__ ?? {};
const invoke = tauri.invoke ?? (async () => { throw new Error('Tauri not available'); });
const listen = tauri.event?.listen?.bind(tauri.event) ?? null;

const overlayRoot = document.getElementById('overlay-root');
const profileSelect = document.getElementById('profile-select');
const portSelect = document.getElementById('port-select');
const startBtn = document.getElementById('start-btn');
const stopBtn = document.getElementById('stop-btn');
const loadConfigBtn = document.getElementById('load-config-btn');
const configStatus = document.getElementById('config-status');
const adapterStatus = document.getElementById('adapter-status');
const profileBadge = document.getElementById('profile-badge');
const portBadge = document.getElementById('port-badge');
const showOverlayBtn = document.getElementById('show-overlay-btn');
const hideOverlayBtn = document.getElementById('hide-overlay-btn');
const startBrowserBtn = document.getElementById('start-browser-btn');
const copyUrlBtn = document.getElementById('copy-url-btn');
const browserUrl = document.getElementById('browser-url');
const syncBtn = document.getElementById('sync-btn');

const overlayHandle = createOverlay(overlayRoot, overlayTemplate);

const state = {
  inputReport: null,
  config: buildDefaultConfig(),
  mapping: null,
  selectedProfile: 0,
  selectedPort: 0,
  overlayUrl: ''
};

function setStatus(el, ok, text) {
  el.classList.remove('ok', 'warn');
  el.classList.add(ok ? 'ok' : 'warn');
  const dot = el.querySelector('.status-dot');
  if (dot) {
    dot.style.background = ok ? 'var(--ok)' : 'var(--warn)';
  }
  const label = el.querySelector('.status-text');
  if (label) {
    label.textContent = text;
  }
}

function storeSelection() {
  localStorage.setItem('orca.viewer.profile', String(state.selectedProfile));
  localStorage.setItem('orca.viewer.port', String(state.selectedPort));
}

function loadSelection() {
  const profile = Number(localStorage.getItem('orca.viewer.profile'));
  const port = Number(localStorage.getItem('orca.viewer.port'));
  if (Number.isFinite(profile) && profile >= 0) state.selectedProfile = profile;
  if (Number.isFinite(port) && port >= 0) state.selectedPort = port;
}

function updateProfileOptions() {
  profileSelect.innerHTML = '';
  const labels = state.config.draft.profileLabels;
  labels.forEach((label, index) => {
    const option = document.createElement('option');
    option.value = String(index);
    option.textContent = label || `Profile ${index + 1}`;
    profileSelect.appendChild(option);
  });
  profileSelect.value = String(state.selectedProfile);
  const badgeValue = profileBadge.querySelector('.badge-value');
  if (badgeValue) badgeValue.textContent = String(state.selectedProfile + 1);
}

function updatePortOptions() {
  portSelect.value = String(state.selectedPort);
  const badgeValue = portBadge.querySelector('.badge-value');
  if (badgeValue) badgeValue.textContent = String(state.selectedPort + 1);
}

function applyConfig(config, source) {
  state.config = config;
  state.selectedProfile = config.header.activeProfile ?? state.selectedProfile;
  updateProfileOptions();
  state.mapping = computeMapping(state.config, state.selectedProfile);
  configStatus.textContent = source === 'device' ? 'Config loaded from device.' : 'Using default Orca mapping.';
  storeSelection();
}

function loadConfigFromStorage() {
  const blobBase64 = localStorage.getItem('orca.viewer.configBlob');
  if (!blobBase64) return false;
  const bytes = decodeBase64ToBytes(blobBase64);
  const parsed = tryParseSettingsBlob(bytes);
  if (!parsed.ok) return false;
  applyConfig(parsed.value, 'device');
  void invoke('set_config_blob', { blob_base64: blobBase64 }).catch(() => {});
  return true;
}

function updateOverlayView() {
  if (!state.mapping) {
    state.mapping = computeMapping(state.config, state.selectedProfile);
  }
  const portData = state.inputReport?.ports?.[state.selectedPort] ?? null;
  updateOverlay(overlayHandle, state.mapping, portData);

  const connected = portData?.connected ?? false;
  if (connected) {
    setStatus(adapterStatus, true, 'Adapter connected');
  } else {
    setStatus(adapterStatus, false, 'Waiting for adapter');
  }
}

async function startStream() {
  try {
    await invoke('start_adapter_stream');
  } catch (error) {
    setStatus(adapterStatus, false, error?.message ?? String(error));
  }
}

async function stopStream() {
  try {
    await invoke('stop_adapter_stream');
  } catch (error) {
    setStatus(adapterStatus, false, error?.message ?? String(error));
  }
}

async function loadConfigFromDevice() {
  configStatus.textContent = 'Waiting for Orca config mode...';
  try {
    const result = await invoke('load_config');
    const blobBase64 = result?.blob_base64 ?? result?.blobBase64;
    if (!blobBase64) throw new Error('No config data returned');

    localStorage.setItem('orca.viewer.configBlob', blobBase64);
    await invoke('set_config_blob', { blob_base64: blobBase64 }).catch(() => {});
    const bytes = decodeBase64ToBytes(blobBase64);
    const parsed = tryParseSettingsBlob(bytes);
    if (!parsed.ok) throw new Error(parsed.error);
    applyConfig(parsed.value, 'device');
    await startStream();
  } catch (error) {
    configStatus.textContent = error?.message ?? String(error);
  }
}

async function startBrowserOverlay() {
  try {
    const info = await invoke('start_overlay_server');
    state.overlayUrl = info?.url ?? '';
    browserUrl.textContent = state.overlayUrl ? state.overlayUrl : 'Overlay server started.';
  } catch (error) {
    browserUrl.textContent = error?.message ?? String(error);
  }
}

async function copyOverlayUrl() {
  if (!state.overlayUrl) return;
  try {
    await navigator.clipboard.writeText(state.overlayUrl);
    browserUrl.textContent = `Copied: ${state.overlayUrl}`;
  } catch {
    browserUrl.textContent = 'Copy failed; select the URL manually.';
  }
}

startBtn.addEventListener('click', () => void startStream());
stopBtn.addEventListener('click', () => void stopStream());
loadConfigBtn.addEventListener('click', () => void loadConfigFromDevice());
showOverlayBtn.addEventListener('click', () => void invoke('show_overlay_window'));
hideOverlayBtn.addEventListener('click', () => void invoke('hide_overlay_window'));
startBrowserBtn.addEventListener('click', () => void startBrowserOverlay());
copyUrlBtn.addEventListener('click', () => void copyOverlayUrl());
syncBtn.addEventListener('click', () => void startStream());

  profileSelect.addEventListener('change', () => {
  state.selectedProfile = Number(profileSelect.value) || 0;
  state.mapping = computeMapping(state.config, state.selectedProfile);
  const badgeValue = profileBadge.querySelector('.badge-value');
  if (badgeValue) badgeValue.textContent = String(state.selectedProfile + 1);
  storeSelection();
  updateOverlayView();
});

portSelect.addEventListener('change', () => {
  state.selectedPort = Number(portSelect.value) || 0;
  const badgeValue = portBadge.querySelector('.badge-value');
  if (badgeValue) badgeValue.textContent = String(state.selectedPort + 1);
  storeSelection();
  updateOverlayView();
});

async function init() {
  loadSelection();
  updateProfileOptions();
  updatePortOptions();

  const restored = loadConfigFromStorage();
  if (!restored) {
    applyConfig(buildDefaultConfig(), 'default');
  }

  if (listen) {
    await listen('input_report', (event) => {
      state.inputReport = event.payload;
      updateOverlayView();
    });
    await listen('adapter_error', (event) => {
      setStatus(adapterStatus, false, event.payload ?? 'Adapter error');
    });
  }

  await startStream();
  updateOverlayView();
}

void init();
