import { applyState, buildDiagram, computeViewerState, decodeConfig } from './lib/orca-viewer.js';

const svg = document.getElementById('orcaDiagram');
let config = decodeConfig(null);
let selectedProfile = config.activeProfile ?? 0;

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

async function pollState() {
  try {
    const res = await fetch('/state');
    if (!res.ok) throw new Error('state failed');
    const data = await res.json();
    if (data.blobBase64) {
      config = decodeConfig(data.blobBase64);
      selectedProfile = config.activeProfile ?? 0;
    }
    render(data.input?.ports ? data.input.ports[0] : data.input);
  } catch (err) {
    console.warn(err);
  } finally {
    setTimeout(pollState, 100);
  }
}

async function bootstrap() {
  if (window.__TAURI__) {
    await loadConfigFromBackend();
    await tauriListen('input_report', (event) => render(event.payload));
  } else {
    pollState();
  }
}

bootstrap();
