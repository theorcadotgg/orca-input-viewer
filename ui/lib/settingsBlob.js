import {
  ORCA_CONFIG_SETTINGS_HEADER_ACTIVE_PROFILE_OFFSET,
  OrcaSettingsTlv
} from './constants.js';
import { decodeNullTerminatedAscii, readF32Le, readU16Le, readU32Le } from './bytes.js';

function readTlvData(blob, tlv, index) {
  const off = tlv.offset0 + index * tlv.stride;
  if (off + 4 + tlv.length > blob.length) {
    throw new Error(`TLV out of range (type=${tlv.type}, index=${index})`);
  }
  const gotType = readU16Le(blob, off);
  const gotLen = readU16Le(blob, off + 2);
  if (gotType !== tlv.type || gotLen !== tlv.length) {
    throw new Error(`Bad TLV header (type=${tlv.type}, index=${index})`);
  }
  return blob.slice(off + 4, off + 4 + tlv.length);
}

function parseProfileLabel(bytes) {
  return decodeNullTerminatedAscii(bytes);
}

function parseTriggerPolicy(data) {
  if (data.length !== OrcaSettingsTlv.TriggerPolicy.length) {
    throw new Error('Bad TriggerPolicy length');
  }
  return {
    analogRangeMax: readF32Le(data, 0),
    digitalFullPress: readF32Le(data, 4),
    digitalLightshield: readF32Le(data, 8),
    flags: data[12] ?? 0,
    digitalLightLtSrc: data[13] ?? 0,
    digitalLightRtSrc: data[14] ?? 0,
    digitalLightSrcVersion: data[15] ?? 0
  };
}

export function parseSettingsBlob(blob) {
  const activeProfile = blob[ORCA_CONFIG_SETTINGS_HEADER_ACTIVE_PROFILE_OFFSET] ?? 0;

  const profileLabels = [];
  for (let i = 0; i < OrcaSettingsTlv.ProfileLabels.count; i++) {
    const data = readTlvData(blob, OrcaSettingsTlv.ProfileLabels, i);
    profileLabels.push(parseProfileLabel(data));
  }

  const digitalMappings = [];
  for (let i = 0; i < OrcaSettingsTlv.DigitalMappings.count; i++) {
    const data = readTlvData(blob, OrcaSettingsTlv.DigitalMappings, i);
    digitalMappings.push(Array.from(data));
  }

  const analogMappings = [];
  for (let i = 0; i < OrcaSettingsTlv.AnalogMappings.count; i++) {
    const data = readTlvData(blob, OrcaSettingsTlv.AnalogMappings, i);
    analogMappings.push(Array.from(data));
  }

  const triggerPolicy = [];
  for (let i = 0; i < OrcaSettingsTlv.TriggerPolicy.count; i++) {
    const data = readTlvData(blob, OrcaSettingsTlv.TriggerPolicy, i);
    triggerPolicy.push(parseTriggerPolicy(data));
  }

  return {
    header: { activeProfile },
    draft: {
      activeProfile,
      profileLabels,
      digitalMappings,
      analogMappings,
      triggerPolicy
    }
  };
}

export function tryParseSettingsBlob(blob) {
  try {
    return { ok: true, value: parseSettingsBlob(blob) };
  } catch (error) {
    return { ok: false, error: error instanceof Error ? error.message : String(error) };
  }
}

export function buildDefaultConfig() {
  const profileLabels = Array.from({ length: 8 }, (_, i) => `Profile ${i + 1}`);
  const digitalMappings = Array.from({ length: 8 }, () =>
    Array.from({ length: 17 }, (_, i) => i)
  );
  const analogMappings = Array.from({ length: 8 }, () =>
    Array.from({ length: 5 }, (_, i) => i)
  );
  const triggerPolicy = Array.from({ length: 8 }, () => ({
    analogRangeMax: 200 / 255,
    digitalFullPress: 200 / 255,
    digitalLightshield: 49 / 255,
    flags: 0,
    digitalLightLtSrc: 0,
    digitalLightRtSrc: 0,
    digitalLightSrcVersion: 0
  }));

  return {
    header: { activeProfile: 0 },
    draft: {
      activeProfile: 0,
      profileLabels,
      digitalMappings,
      analogMappings,
      triggerPolicy
    }
  };
}

export function decodeBase64ToBytes(base64) {
  const bin = atob(base64);
  const bytes = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) {
    bytes[i] = bin.charCodeAt(i);
  }
  return bytes;
}

export function encodeBytesToBase64(bytes) {
  let out = '';
  for (let i = 0; i < bytes.length; i++) {
    out += String.fromCharCode(bytes[i]);
  }
  return btoa(out);
}
