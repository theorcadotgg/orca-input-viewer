import { applyState, buildDiagram, computeViewerState, decodeConfig } from './lib/orca-viewer.js';

const svg = document.getElementById('orcaDiagram');
const toggleChromaBtn = document.getElementById('toggleChroma');
const togglePinBtn = document.getElementById('togglePin');
const closeOverlayBtn = document.getElementById('closeOverlay');

let config = decodeConfig(null);
let selectedProfile = config.activeProfile ?? 0;
let isPinned = false;
let ws = null;

buildDiagram(svg);

function tauriInvoke(command, args = {}) {
  const tauri = window.__TAURI__;
  if (!tauri) return Promise.reject(new Error('Tauri not available'));
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

function render(report) {
  const portReport = report?.ports?.find((p) => p.port === 0) ?? report;
  const state = computeViewerState(portReport, config, selectedProfile);
  applyState(svg, state);
}

async function loadConfigFromBackend() {
  try {
    const blob = await tauriInvoke('get_config_blob');
    if (blob) {
      config = decodeConfig(blob);
      selectedProfile = config.activeProfile ?? 0;
      render(null);
    }
  } catch (err) {
    console.warn(err);
  }
}

// WebSocket-based state updates (much faster than HTTP polling)
function connectWebSocket(wsUrl) {
  if (ws) {
    ws.close();
  }

  ws = new WebSocket(wsUrl);

  ws.onopen = () => {
    console.log('WebSocket connected');
  };

  ws.onmessage = (event) => {
    try {
      const data = JSON.parse(event.data);
      if (data.blobBase64) {
        config = decodeConfig(data.blobBase64);
      }
      if (typeof data.profile === 'number') {
        selectedProfile = data.profile;
      } else {
        selectedProfile = config.activeProfile ?? 0;
      }
      render(data.input?.ports ? data.input.ports[0] : data.input);
    } catch (err) {
      console.warn('WebSocket message parse error:', err);
    }
  };

  ws.onclose = () => {
    console.log('WebSocket closed, reconnecting...');
    setTimeout(() => connectWebSocket(wsUrl), 1000);
  };

  ws.onerror = (err) => {
    console.warn('WebSocket error:', err);
  };
}

// Fallback HTTP polling for browser source mode
async function pollState() {
  try {
    const res = await fetch('/state');
    if (!res.ok) throw new Error('state failed');
    const data = await res.json();
    if (data.blobBase64) {
      config = decodeConfig(data.blobBase64);
    }
    if (typeof data.profile === 'number') {
      selectedProfile = data.profile;
    } else {
      selectedProfile = config.activeProfile ?? 0;
    }
    render(data.input?.ports ? data.input.ports[0] : data.input);
  } catch (err) {
    console.warn(err);
  } finally {
    setTimeout(pollState, 16); // ~60fps polling as fallback
  }
}

// Try to get WebSocket port and connect
async function tryWebSocketConnection() {
  try {
    const res = await fetch('/ws-port');
    if (res.ok) {
      const data = await res.json();
      if (data.port) {
        const wsUrl = `ws://127.0.0.1:${data.port}`;
        connectWebSocket(wsUrl);
        return true;
      }
    }
  } catch (err) {
    console.warn('Could not get WebSocket port:', err);
  }
  return false;
}

// Overlay controls
if (toggleChromaBtn) {
  toggleChromaBtn.addEventListener('click', () => {
    document.body.classList.toggle('chroma-mode');
    toggleChromaBtn.classList.toggle('active', document.body.classList.contains('chroma-mode'));
  });
}

if (togglePinBtn) {
  togglePinBtn.addEventListener('click', async () => {
    isPinned = !isPinned;
    togglePinBtn.classList.toggle('active', isPinned);
    try {
      await tauriInvoke('set_overlay_always_on_top', { enabled: isPinned });
    } catch (err) {
      console.warn('Failed to toggle always on top:', err);
    }
  });
}

if (closeOverlayBtn) {
  closeOverlayBtn.addEventListener('click', async () => {
    try {
      await tauriInvoke('hide_overlay_window');
    } catch (err) {
      console.warn('Failed to hide overlay:', err);
    }
  });
}

async function bootstrap() {
  if (window.__TAURI__) {
    // Running as Tauri window - use native events
    await loadConfigFromBackend();
    await tauriListen('input_report', (event) => render(event.payload));

    // Listen for config changes from main window
    await tauriListen('config_changed', (event) => {
      const { blobBase64, profile } = event.payload;
      if (blobBase64) {
        config = decodeConfig(blobBase64);
      }
      if (typeof profile === 'number') {
        selectedProfile = profile;
      }
      render(null);
    });

    // Listen for profile selection changes from main window
    await tauriListen('profile_changed', (event) => {
      const { profile } = event.payload;
      if (typeof profile === 'number') {
        selectedProfile = profile;
        render(null);
      }
    });
  } else {
    // Running as browser source - try WebSocket first, fallback to polling
    const wsConnected = await tryWebSocketConnection();
    if (!wsConnected) {
      console.log('WebSocket not available, falling back to HTTP polling');
      pollState();
    }
  }
}

bootstrap();
