export const ORCA_CONFIG_SCHEMA_ID = 0x6c42d6ba;
export const ORCA_CONFIG_SETTINGS_VERSION_MAJOR = 2;
export const ORCA_CONFIG_SETTINGS_VERSION_MINOR = 7;
export const ORCA_CONFIG_SETTINGS_BLOB_SIZE = 16384;
export const ORCA_CONFIG_SETTINGS_PROFILE_COUNT = 8;

export const ORCA_CONFIG_ORCA_DIGITAL_INPUT_COUNT = 17;
export const ORCA_CONFIG_ORCA_ANALOG_INPUT_COUNT = 5;

export const ORCA_CONFIG_LOCKED_BUTTON_WISDOM = 13;
export const ORCA_CONFIG_LOCKED_BUTTON_COURAGE = 14;
export const ORCA_CONFIG_LOCKED_BUTTON_POWER = 15;

export const ORCA_CONFIG_SETTINGS_HEADER_MAGIC_OFFSET = 0;
export const ORCA_CONFIG_SETTINGS_HEADER_VERSION_MAJOR_OFFSET = 16;
export const ORCA_CONFIG_SETTINGS_HEADER_VERSION_MINOR_OFFSET = 17;
export const ORCA_CONFIG_SETTINGS_HEADER_HEADER_SIZE_OFFSET = 18;
export const ORCA_CONFIG_SETTINGS_HEADER_GENERATION_OFFSET = 20;
export const ORCA_CONFIG_SETTINGS_HEADER_ACTIVE_PROFILE_OFFSET = 24;
export const ORCA_CONFIG_SETTINGS_HEADER_FLAGS_OFFSET = 25;

export const OrcaSettingsTlv = {
  RangeCalibration: { type: 0, length: 44, count: 1, stride: 48, offset0: 128 },
  DeadzoneCalibration: { type: 1, length: 24, count: 2, stride: 28, offset0: 176 },
  NotchCalibration: { type: 2, length: 24, count: 1, stride: 28, offset0: 232 },
  ProfileLabels: { type: 3, length: 32, count: 8, stride: 36, offset0: 260 },
  DigitalMappings: { type: 4, length: 17, count: 8, stride: 22, offset0: 548 },
  AnalogMappings: { type: 5, length: 5, count: 8, stride: 10, offset0: 724 },
  StickCurveParams: { type: 6, length: 96, count: 8, stride: 100, offset0: 804 },
  DpadLayer: { type: 7, length: 64, count: 8, stride: 68, offset0: 1604 },
  TriggerPolicy: { type: 8, length: 16, count: 8, stride: 20, offset0: 2148 }
};
