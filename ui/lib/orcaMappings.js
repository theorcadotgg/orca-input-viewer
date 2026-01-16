import {
  ORCA_CONFIG_LOCKED_BUTTON_COURAGE,
  ORCA_CONFIG_LOCKED_BUTTON_POWER,
  ORCA_CONFIG_LOCKED_BUTTON_WISDOM,
  ORCA_CONFIG_ORCA_ANALOG_INPUT_COUNT,
  ORCA_CONFIG_ORCA_DIGITAL_INPUT_COUNT
} from './constants.js';

export const ORCA_DUMMY_FIELD = ORCA_CONFIG_ORCA_DIGITAL_INPUT_COUNT - 1;
export const ORCA_ANALOG_MAPPING_DISABLED = 0xff;

export const DIGITAL_INPUTS = [
  { id: 0, label: 'A' },
  { id: 1, label: 'B' },
  { id: 2, label: 'X' },
  { id: 3, label: 'Y' },
  { id: 4, label: 'Z' },
  { id: 5, label: 'L' },
  { id: 6, label: 'R' },
  { id: 7, label: 'C Left' },
  { id: 8, label: 'C Right' },
  { id: 9, label: 'C Up' },
  { id: 10, label: 'C Down' },
  { id: 11, label: 'DPAD Mod' },
  { id: 12, label: 'Lightshield' },
  { id: ORCA_CONFIG_LOCKED_BUTTON_WISDOM, label: 'Start' },
  { id: ORCA_CONFIG_LOCKED_BUTTON_COURAGE, label: 'Courage' },
  { id: ORCA_CONFIG_LOCKED_BUTTON_POWER, label: 'Power' },
  { id: ORCA_DUMMY_FIELD, label: 'Disabled' }
];

export const ANALOG_INPUTS = [
  { id: 0, label: 'Stick Left' },
  { id: 1, label: 'Stick Right' },
  { id: 2, label: 'Stick Up' },
  { id: 3, label: 'Stick Down' },
  { id: 4, label: 'Trigger R' }
];

export function digitalInputLabel(id) {
  return DIGITAL_INPUTS.find((d) => d.id === id)?.label ?? `Digital ${id}`;
}

export function analogInputLabel(id) {
  return ANALOG_INPUTS.find((d) => d.id === id)?.label ?? `Analog ${id}`;
}

export function digitalOutputShortLabel(id) {
  switch (id) {
    case 0: return 'A';
    case 1: return 'B';
    case 2: return 'X';
    case 3: return 'Y';
    case 4: return 'Z';
    case 5: return 'L';
    case 6: return 'R';
    case 7: return 'CL';
    case 8: return 'CR';
    case 9: return 'CU';
    case 10: return 'CD';
    case 11: return 'DP';
    case 12: return 'LS';
    case 13: return 'ST';
    case 14: return 'CG';
    case 15: return 'PW';
    default: return 'OFF';
  }
}

export function analogOutputShortLabel(id) {
  switch (id) {
    case 0: return 'L';
    case 1: return 'R';
    case 2: return 'U';
    case 3: return 'D';
    case 4: return 'TR';
    default: return 'OFF';
  }
}
