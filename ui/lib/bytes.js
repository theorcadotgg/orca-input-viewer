export function readU16Le(bytes, offset) {
  return bytes[offset] | (bytes[offset + 1] << 8);
}

export function readU32Le(bytes, offset) {
  return (bytes[offset] | (bytes[offset + 1] << 8) | (bytes[offset + 2] << 16) | (bytes[offset + 3] << 24)) >>> 0;
}

export function readF32Le(bytes, offset) {
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  return view.getFloat32(offset, true);
}

export function decodeNullTerminatedAscii(bytes) {
  let end = bytes.indexOf(0);
  if (end === -1) end = bytes.length;
  let out = '';
  for (let i = 0; i < end; i++) {
    out += String.fromCharCode(bytes[i]);
  }
  return out;
}
