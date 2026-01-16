import { ORCA_ANALOG_MAPPING_DISABLED, ORCA_DUMMY_FIELD, digitalOutputShortLabel, analogOutputShortLabel } from './orcaMappings.js';

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
  { id: 13, elementIndex: 14 }
];

const ANALOG_BUTTONS = [
  { id: 0, elementIndex: 2 },
  { id: 1, elementIndex: 0 },
  { id: 2, elementIndex: 1 },
  { id: 3, elementIndex: 3 },
  { id: 4, elementIndex: 4 }
];

const circleIndexToSource = new Map(DIGITAL_BUTTONS.map((b) => [b.elementIndex, b.id]));
const oblongIndexToSource = new Map(ANALOG_BUTTONS.map((b) => [b.elementIndex, b.id]));

function clamp01(value) {
  if (!Number.isFinite(value)) return 0;
  return Math.min(1, Math.max(0, value));
}

function clampSigned(value) {
  if (!Number.isFinite(value)) return 0;
  return Math.min(1, Math.max(-1, value));
}

function buildOutputState(input) {
  const digital = new Array(17).fill(false);
  const analog = new Array(5).fill(0);

  if (!input || !input.connected) {
    return { digital, analog };
  }

  digital[0] = !!input.buttons.a;
  digital[1] = !!input.buttons.b;
  digital[2] = !!input.buttons.x;
  digital[3] = !!input.buttons.y;
  digital[4] = !!input.buttons.z;
  digital[5] = !!input.buttons.l;
  digital[6] = !!input.buttons.r;
  digital[13] = !!input.buttons.start;

  const cThresh = 0.45;
  if (input.axes.substick_x < -cThresh) digital[7] = true;
  if (input.axes.substick_x > cThresh) digital[8] = true;
  if (input.axes.substick_y > cThresh) digital[9] = true;
  if (input.axes.substick_y < -cThresh) digital[10] = true;

  const stickX = clampSigned(input.axes.stick_x);
  const stickY = clampSigned(input.axes.stick_y);

  analog[0] = clamp01(-stickX);
  analog[1] = clamp01(stickX);
  analog[2] = clamp01(stickY);
  analog[3] = clamp01(-stickY);
  analog[4] = clamp01(Math.max(input.axes.trigger_l, input.axes.trigger_r));

  return { digital, analog };
}

export function createOverlay(root, template) {
  root.innerHTML = template;
  const digitalGroups = Array.from(root.querySelectorAll('[data-digital-index]')).map((group) => ({
    elementIndex: Number(group.dataset.digitalIndex),
    circle: group.querySelector('circle'),
    label: group.querySelector('text')
  }));

  const analogGroups = Array.from(root.querySelectorAll('[data-analog-index]')).map((group) => ({
    elementIndex: Number(group.dataset.analogIndex),
    path: group.querySelector('path'),
    label: group.querySelector('.oblong-label'),
    value: group.querySelector('.oblong-value')
  }));

  return { digitalGroups, analogGroups };
}

export function computeMapping(config, profileIndex) {
  const draft = config.draft;
  const digitalMapping = draft.digitalMappings[profileIndex] ?? draft.digitalMappings[0];
  const analogMapping = draft.analogMappings[profileIndex] ?? draft.analogMappings[0];
  const triggerPolicy = draft.triggerPolicy[profileIndex] ?? draft.triggerPolicy[0];

  const digitalDestBySrc = Array.from({ length: 17 }, () => ORCA_DUMMY_FIELD);
  for (let dest = 0; dest < digitalMapping.length; dest++) {
    const src = digitalMapping[dest];
    if (src === ORCA_DUMMY_FIELD) continue;
    if (src >= 0 && src < digitalDestBySrc.length) {
      digitalDestBySrc[src] = dest;
    }
  }

  const analogDestBySrc = Array.from({ length: 5 }, () => ORCA_ANALOG_MAPPING_DISABLED);
  for (let dest = 0; dest < analogMapping.length; dest++) {
    const src = analogMapping[dest];
    if (src === ORCA_ANALOG_MAPPING_DISABLED) continue;
    if (src >= 0 && src < analogDestBySrc.length) {
      analogDestBySrc[src] = dest;
    }
  }

  return { digitalDestBySrc, analogDestBySrc, triggerPolicy };
}

export function updateOverlay(handle, mapping, input) {
  const output = buildOutputState(input);

  handle.digitalGroups.forEach((group) => {
    const srcId = circleIndexToSource.get(group.elementIndex);
    if (srcId === undefined) return;
    const dest = mapping.digitalDestBySrc[srcId] ?? ORCA_DUMMY_FIELD;
    const label = digitalOutputShortLabel(dest);
    const pressed = dest !== ORCA_DUMMY_FIELD && output.digital[dest];

    group.label.textContent = label;
    group.circle.classList.toggle('active', pressed);
    group.circle.classList.toggle('off', dest === ORCA_DUMMY_FIELD);
  });

  const light = mapping.triggerPolicy?.digitalLightshield ?? 0.19;
  const full = mapping.triggerPolicy?.digitalFullPress ?? 0.78;
  const mid = (light + full) / 2;

  handle.analogGroups.forEach((group) => {
    const srcId = oblongIndexToSource.get(group.elementIndex);
    if (srcId === undefined) return;
    const dest = mapping.analogDestBySrc[srcId] ?? ORCA_ANALOG_MAPPING_DISABLED;
    const label = analogOutputShortLabel(dest);
    const value = dest !== ORCA_ANALOG_MAPPING_DISABLED ? output.analog[dest] : 0;

    group.label.textContent = label;
    group.value.textContent = value.toFixed(2);

    group.path.classList.remove('light', 'mid', 'full');
    if (value > 0.01) {
      if (value < light) {
        group.path.classList.add('light');
      } else if (value < full) {
        group.path.classList.add('mid');
      } else {
        group.path.classList.add('full');
      }
    }
  });
}
