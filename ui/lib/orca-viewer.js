const ORCA_DUMMY_FIELD = 16;
const ORCA_ANALOG_DISABLED = 0xff;

const HEADER_ACTIVE_PROFILE_OFFSET = 24;

const TLV = {
  ProfileLabels: { type: 3, length: 32, count: 8, stride: 36, offset0: 260 },
  DigitalMappings: { type: 4, length: 17, count: 8, stride: 22, offset0: 548 },
  AnalogMappings: { type: 5, length: 5, count: 8, stride: 10, offset0: 724 },
  TriggerPolicy: { type: 8, length: 16, count: 8, stride: 20, offset0: 2148 },
};

const DIGITAL_INPUTS = [
  { id: 0, label: 'A', shortLabel: 'A' },
  { id: 1, label: 'B', shortLabel: 'B' },
  { id: 2, label: 'X', shortLabel: 'X' },
  { id: 3, label: 'Y', shortLabel: 'Y' },
  { id: 4, label: 'Z', shortLabel: 'Z' },
  { id: 5, label: 'L', shortLabel: 'L' },
  { id: 6, label: 'R', shortLabel: 'R' },
  { id: 7, label: 'C Left', shortLabel: 'C<' },
  { id: 8, label: 'C Right', shortLabel: 'C>' },
  { id: 9, label: 'C Up', shortLabel: 'C^' },
  { id: 10, label: 'C Down', shortLabel: 'Cv' },
  { id: 11, label: 'Dpad', shortLabel: 'D' },
  { id: 12, label: 'Lightshield', shortLabel: 'LS' },
  { id: 13, label: 'Start', shortLabel: 'St' },
  { id: 14, label: 'Courage', shortLabel: 'Sys' },
  { id: 15, label: 'Power', shortLabel: 'Pwr' },
  { id: 16, label: 'Off', shortLabel: 'Off' },
];

const ANALOG_INPUTS = [
  { id: 0, label: 'Stick Left', shortLabel: '<' },
  { id: 1, label: 'Stick Right', shortLabel: '>' },
  { id: 2, label: 'Stick Up', shortLabel: '^' },
  { id: 3, label: 'Stick Down', shortLabel: 'v' },
  { id: 4, label: 'Trigger R', shortLabel: 'TR' },
];

const DIGITAL_BUTTONS = [
  { id: 0, elementIndex: 8 },
  { id: 1, elementIndex: 11 },
  { id: 2, elementIndex: 9 },
  { id: 3, elementIndex: 13 },
  { id: 4, elementIndex: 10 },
  { id: 5, elementIndex: 2 },
  { id: 6, elementIndex: 0 },
  { id: 7, elementIndex: 6 },
  { id: 8, elementIndex: 1 },
  { id: 9, elementIndex: 7 },
  { id: 10, elementIndex: 5 },
  { id: 11, elementIndex: 3 },
  { id: 12, elementIndex: 4 },
  { id: 13, elementIndex: 14 },
];

const ANALOG_BUTTONS = [
  { id: 0, elementIndex: 2 },
  { id: 1, elementIndex: 0 },
  { id: 2, elementIndex: 1 },
  { id: 3, elementIndex: 3 },
  { id: 4, elementIndex: 4 },
];

const CIRCLES = [
  { cx: 219.645, cy: 229.0637, r: 11.5 },
  { cx: 224.98, cy: 294.6787, r: 11.5 },
  { cx: 21.735, cy: 256.7337, r: 11.5 },
  { cx: 178.5, cy: 251.4037, r: 11.5 },
  { cx: 265.18, cy: 214.2387, r: 11.5 },
  { cx: 199.0, cy: 339.7187, r: 11.5 },
  { cx: 185.995, cy: 317.1937, r: 11.5 },
  { cx: 198.98, cy: 294.6787, r: 11.5 },
  { cx: 212.0, cy: 317.1987, r: 11.5 },
  { cx: 244.57, cy: 237.1287, r: 11.5 },
  { cx: 270.63, cy: 239.8687, r: 11.5 },
  { cx: 224.98, cy: 254.6787, r: 11.5 },
  { cx: 165.665, cy: 229.0637, r: 11.5 },
  { cx: 239.12, cy: 211.4987, r: 11.5 },
  { cx: 152.5, cy: 251.4787, r: 11.5 },
];

const OBLONGS = [
  { cx: 110, cy: 242, path: 'M124.6254 228.8973 A12 12 0 1 0 101.1499 223.9075 L94.7046 254.23 A12 12 0 1 0 118.1802 259.2199 Z' },
  { cx: 84, cy: 228, path: 'M98.7154 213.6873 A12 12 0 1 0 75.2399 208.6975 L68.7946 239.02 A12 12 0 1 0 92.2702 244.0099 Z' },
  { cx: 54, cy: 230, path: 'M68.8654 217.0673 A12 12 0 1 0 45.3899 212.0775 L38.9446 242.4 A12 12 0 1 0 62.4202 247.3899 Z' },
  { cx: 123, cy: 318, path: 'M141.611 311.0539 A12 12 0 0 0 121.7141 297.6333 L104.3791 323.3334 A12 12 0 0 0 124.276 336.7541 Z' },
  { cx: 300, cy: 248, path: 'M309.868 229.2229 A12 12 0 1 0 286.2326 233.3905 L291.6156 263.9195 A12 12 0 1 0 315.251 259.752 Z' },
];

// Single continuous path for controller outline (clockwise from top-left)
const CONTROLLER_OUTLINE = `
  M64.037 184.995
  L267.163 184.995
  A16 16 0 0 1 275.151 187.132
  L326.093 216.485
  A10 10 0 0 1 331.1 225.15
  L331.1 329.841
  A10 10 0 0 1 326.093 338.505
  L275.151 367.858
  A16 16 0 0 1 267.163 369.995
  L64.037 369.995
  A16 16 0 0 1 56.049 367.858
  L5.107 338.505
  A10 10 0 0 1 0.1 329.841
  L0.1 225.15
  A10 10 0 0 1 5.107 216.485
  L56.049 187.132
  A16 16 0 0 1 64.037 184.995
  Z
`;

const ADAPTER_BUTTON_ORDER = [
  'a',
  'b',
  'x',
  'y',
  'dpad_left',
  'dpad_right',
  'dpad_down',
  'dpad_up',
  'start',
  'z',
  'r',
  'l',
];

const INTERMEDIATE_USB_ADAPTER_DIGITAL_MAPPING = [
  0,
  1,
  2,
  3,
  16,
  17,
  15,
  14,
  11,
  4,
  8,
  5,
];

const ORCA_INTERMEDIATE_DIGITAL_MAPPING = [
  0,
  1,
  2,
  3,
  4,
  5,
  16,
  16,
  6,
  16,
  16,
  15,
  13,
  14,
  16,
  16,
  16,
  16,
];

function readU16(data, offset) {
  return data[offset] | (data[offset + 1] << 8);
}

function readU32(data, offset) {
  return (data[offset] | (data[offset + 1] << 8) | (data[offset + 2] << 16) | (data[offset + 3] << 24)) >>> 0;
}

function readF32(data, offset) {
  const dv = new DataView(data.buffer, data.byteOffset + offset, 4);
  return dv.getFloat32(0, true);
}

function readTlv(data, tlv, index) {
  const off = tlv.offset0 + index * tlv.stride;
  const gotType = readU16(data, off);
  const gotLen = readU16(data, off + 2);
  if (gotType !== tlv.type || gotLen !== tlv.length) {
    return null;
  }
  return data.slice(off + 4, off + 4 + tlv.length);
}

function decodeLabel(bytes) {
  let out = '';
  for (let i = 0; i < bytes.length; i++) {
    const c = bytes[i];
    if (!c) break;
    out += String.fromCharCode(c);
  }
  return out || 'Profile';
}

export function decodeConfig(blobBase64) {
  if (!blobBase64) {
    return defaultConfig();
  }

  let raw;
  try {
    raw = Uint8Array.from(atob(blobBase64), (c) => c.charCodeAt(0));
  } catch (err) {
    return defaultConfig();
  }

  const activeProfile = raw[HEADER_ACTIVE_PROFILE_OFFSET] ?? 0;
  const profileLabels = [];
  const digitalMappings = [];
  const analogMappings = [];
  const triggerPolicy = [];

  for (let i = 0; i < TLV.ProfileLabels.count; i++) {
    const data = readTlv(raw, TLV.ProfileLabels, i);
    profileLabels.push(data ? decodeLabel(data) : `Profile ${i + 1}`);
  }

  for (let i = 0; i < TLV.DigitalMappings.count; i++) {
    const data = readTlv(raw, TLV.DigitalMappings, i);
    if (data && data.length === TLV.DigitalMappings.length) {
      digitalMappings.push([...data]);
    } else {
      digitalMappings.push(defaultDigitalMapping());
    }
  }

  for (let i = 0; i < TLV.AnalogMappings.count; i++) {
    const data = readTlv(raw, TLV.AnalogMappings, i);
    if (data && data.length === TLV.AnalogMappings.length) {
      analogMappings.push([...data]);
    } else {
      analogMappings.push(defaultAnalogMapping());
    }
  }

  for (let i = 0; i < TLV.TriggerPolicy.count; i++) {
    const data = readTlv(raw, TLV.TriggerPolicy, i);
    if (data && data.length === TLV.TriggerPolicy.length) {
      triggerPolicy.push({
        analogRangeMax: readF32(data, 0),
        digitalFullPress: readF32(data, 4),
        digitalLightshield: readF32(data, 8),
        flags: data[12] ?? 0,
        digitalLightLtSrc: data[13] ?? 0,
        digitalLightRtSrc: data[14] ?? 0,
        digitalLightSrcVersion: data[15] ?? 0,
      });
    } else {
      triggerPolicy.push(defaultTriggerPolicy());
    }
  }

  return {
    activeProfile: Math.min(activeProfile, 7),
    profileLabels,
    digitalMappings,
    analogMappings,
    triggerPolicy,
  };
}

export function defaultConfig() {
  const profileLabels = Array.from({ length: 8 }, (_, i) => (i === 0 ? 'Default Profile' : `Profile ${i + 1}`));
  const digitalMappings = Array.from({ length: 8 }, () => defaultDigitalMapping());
  const analogMappings = Array.from({ length: 8 }, () => defaultAnalogMapping());
  const triggerPolicy = Array.from({ length: 8 }, () => defaultTriggerPolicy());
  return { activeProfile: 0, profileLabels, digitalMappings, analogMappings, triggerPolicy };
}

function defaultDigitalMapping() {
  return Array.from({ length: 17 }, (_, i) => i);
}

function defaultAnalogMapping() {
  return Array.from({ length: 5 }, (_, i) => i);
}

function defaultTriggerPolicy() {
  return {
    analogRangeMax: 200 / 255,
    digitalFullPress: 200 / 255,
    digitalLightshield: 49 / 255,
    flags: 0,
    digitalLightLtSrc: 0,
    digitalLightRtSrc: 0,
    digitalLightSrcVersion: 0,
  };
}

export function buildDiagram(svg) {
  const parts = [];
  // Draw the controller outline as a single connected path
  parts.push(`<path d="${CONTROLLER_OUTLINE}" class="keyline" />`);


  OBLONGS.forEach((oblong, idx) => {
    parts.push(`\
      <g class="node" data-type="analog" data-index="${idx}">
        <path class="oblong-node" d="${oblong.path}" />
        <text class="node-label" x="${oblong.cx}" y="${oblong.cy}" text-anchor="middle"></text>
        <text class="node-value" x="${oblong.cx}" y="${oblong.cy + 10}" text-anchor="middle"></text>
      </g>`);
  });

  CIRCLES.forEach((circle, idx) => {
    parts.push(`\
      <g class="node" data-type="digital" data-index="${idx}">
        <circle class="circle-node" cx="${circle.cx}" cy="${circle.cy}" r="${circle.r}" />
        <text class="node-label" x="${circle.cx}" y="${circle.cy + 3}" text-anchor="middle"></text>
        <text class="node-value" x="${circle.cx}" y="${circle.cy + 12}" text-anchor="middle"></text>
      </g>`);
  });

  svg.innerHTML = parts.join('\n');
}

export function computeViewerState(report, config, profileIndex) {
  const profile = Math.min(profileIndex ?? config.activeProfile ?? 0, 7);
  const digitalMapping = config.digitalMappings?.[profile] ?? defaultDigitalMapping();
  const analogMapping = config.analogMappings?.[profile] ?? defaultAnalogMapping();
  const triggerPolicy = config.triggerPolicy?.[profile] ?? defaultTriggerPolicy();

  // Physical button states (indexed by physical Orca button position)
  const digitalActiveBySrc = Array.from({ length: 17 }, () => false);
  const digitalValueBySrc = Array.from({ length: 17 }, () => 0);

  // Analog values (indexed by physical analog input position)
  const analogValueBySrc = Array.from({ length: 5 }, () => 0);

  // Labels based on what each physical button outputs (from profile mapping)
  const digitalLabelBySrc = digitalMapping.map(outputId => {
    const entry = DIGITAL_INPUTS.find(d => d.id === outputId);
    return entry ? entry.shortLabel : 'Off';
  });

  const analogLabelBySrc = analogMapping.map(outputId => {
    if (outputId === ORCA_ANALOG_DISABLED) return 'Off';
    const entry = ANALOG_INPUTS.find(a => a.id === outputId);
    return entry ? entry.shortLabel : 'Off';
  });

  if (report) {
    const buttons = report.buttons ?? {};
    const axes = report.axes ?? {};

    // Determine which GCC outputs are active from adapter signals
    // GCC output IDs: 0=A, 1=B, 2=X, 3=Y, 4=Z, 5=L, 6=R, 7=C<, 8=C>, 9=C^, 10=Cv, 11=D, 12=LS, 13=St
    const activeGccOutputs = new Set();

    if (buttons.a) activeGccOutputs.add(0);  // A
    if (buttons.b) activeGccOutputs.add(1);  // B
    if (buttons.x) activeGccOutputs.add(2);  // X
    if (buttons.y) activeGccOutputs.add(3);  // Y
    if (buttons.z || buttons.dpad_right) activeGccOutputs.add(4);  // Z (Orca sends via dpad_right)
    if (buttons.start) activeGccOutputs.add(13);  // Start

    // L/R detection - Orca sends digital L/R via dpad_up/dpad_down
    const digitalLPressed = buttons.l || buttons.dpad_up;
    const digitalRPressed = buttons.r || buttons.dpad_down;

    if (digitalLPressed) activeGccOutputs.add(5);  // L
    if (digitalRPressed) activeGccOutputs.add(6);  // R

    // Lightshield detection from analog trigger (when L not fully pressed)
    const triggerL = axes.trigger_l ?? 0;
    const lightshieldThreshold = 0.15;
    if (!digitalLPressed && triggerL >= lightshieldThreshold) {
      activeGccOutputs.add(12);  // Lightshield
    }

    // C-stick as digital outputs (threshold at 0.5)
    const cstickThreshold = 0.5;
    const substickX = axes.substick_x ?? 0;
    const substickY = axes.substick_y ?? 0;
    if (substickX < -cstickThreshold) activeGccOutputs.add(7);   // C Left
    if (substickX > cstickThreshold) activeGccOutputs.add(8);    // C Right
    if (substickY > cstickThreshold) activeGccOutputs.add(9);    // C Up
    if (substickY < -cstickThreshold) activeGccOutputs.add(10);  // C Down

    // Reverse mapping: for each active GCC output, find physical buttons that produce it
    for (let physicalBtn = 0; physicalBtn < digitalMapping.length; physicalBtn++) {
      const outputId = digitalMapping[physicalBtn];
      if (activeGccOutputs.has(outputId)) {
        digitalActiveBySrc[physicalBtn] = true;
        digitalValueBySrc[physicalBtn] = 1;
      }
    }

    // Analog stick values
    const stickX = axes.stick_x ?? 0;
    const stickY = axes.stick_y ?? 0;

    // Map analog inputs through profile mapping
    // analogMapping[physicalInput] = outputId
    // We need to show the value at the physical position with the remapped label
    const analogValues = [
      Math.max(0, -stickX),  // Physical 0: Stick Left
      Math.max(0, stickX),   // Physical 1: Stick Right
      Math.max(0, stickY),   // Physical 2: Stick Up
      Math.max(0, -stickY),  // Physical 3: Stick Down
      !digitalRPressed ? (axes.trigger_r ?? 0) : 0,  // Physical 4: Trigger R (hide when digital R pressed)
    ];

    // Apply analog values to their physical positions
    for (let i = 0; i < analogValues.length; i++) {
      analogValueBySrc[i] = analogValues[i];
    }
  }

  return {
    digitalActiveBySrc,
    digitalValueBySrc,
    analogValueBySrc,
    digitalLabelBySrc,
    analogLabelBySrc,
    triggerPolicy,
  };
}

export function applyState(svg, state) {
  const nodes = svg.querySelectorAll('.node');
  nodes.forEach((node) => {
    const type = node.getAttribute('data-type');
    const index = Number(node.getAttribute('data-index'));

    if (type === 'digital') {
      const srcId = circleIndexToSource(index);
      const active = state.digitalActiveBySrc[srcId];
      const label = state.digitalLabelBySrc[srcId] ?? '';
      const value = state.digitalValueBySrc[srcId] ?? 0;
      updateNode(node, active ? 1 : 0, label, value, null);
    }

    if (type === 'analog') {
      const srcId = oblongIndexToSource(index);
      const value = state.analogValueBySrc[srcId] ?? 0;
      const label = state.analogLabelBySrc[srcId] ?? '';
      const thresholds = analogThresholds(srcId, state.triggerPolicy);
      updateNode(node, value, label, value, thresholds);
    }
  });
}

function updateNode(node, intensity, label, value, thresholds) {
  node.classList.remove('node-light', 'node-mid', 'node-full');

  if (thresholds) {
    const level = intensityLevel(intensity, thresholds.light, thresholds.mid, thresholds.full);
    if (level) node.classList.add(level);
  } else if (intensity > 0) {
    node.classList.add('node-full');
  }

  const labelEl = node.querySelector('.node-label');
  if (labelEl) labelEl.textContent = label;

  const valueEl = node.querySelector('.node-value');
  if (valueEl) {
    valueEl.textContent = intensity > 0 ? value.toFixed(2) : '';
  }
}

function intensityLevel(value, light, mid, full) {
  if (value >= full) return 'node-full';
  if (value >= mid) return 'node-mid';
  if (value >= light) return 'node-light';
  return '';
}

function analogThresholds(srcId, policy) {
  if (srcId === 4 && policy) {
    const light = policy.digitalLightshield ?? 0.2;
    const full = policy.digitalFullPress ?? 0.8;
    return { light, mid: (light + full) / 2, full };
  }
  return { light: 0.25, mid: 0.55, full: 0.85 };
}

function circleIndexToSource(circleIndex) {
  const match = DIGITAL_BUTTONS.find((b) => b.elementIndex === circleIndex);
  return match ? match.id : ORCA_DUMMY_FIELD;
}

function oblongIndexToSource(oblongIndex) {
  const match = ANALOG_BUTTONS.find((b) => b.elementIndex === oblongIndex);
  return match ? match.id : 0;
}

function digitalLabel(destId) {
  const entry = DIGITAL_INPUTS.find((d) => d.id === destId);
  return entry ? entry.shortLabel : 'Off';
}

function analogLabel(destId) {
  if (destId === ORCA_ANALOG_DISABLED) return 'Off';
  const entry = ANALOG_INPUTS.find((d) => d.id === destId);
  return entry ? entry.shortLabel : 'Off';
}

export function formatPortLabel(port) {
  return `Port ${port + 1}`;
}

export function makePortOptions(count) {
  return Array.from({ length: count }, (_, i) => ({ value: i, label: formatPortLabel(i) }));
}
