/**
 * Convert given string to UTF-8.
 */
export function toUtf8(str: string): Uint8Array {
  return Buffer.from(str, "utf8");
}

/**
 * Convert given UTF-8 to string.
 */
export function fromUtf8(buf: Uint8Array): string {
  return new TextDecoder("utf8").decode(buf);
}

/**
 * Get the length of given string in UTF-8.
 */
export function utf8Length(str: string): number {
  return Buffer.byteLength(str, "utf8");
}

/**
 * Copy given buffer into memory.
 *
 * @returns The number of bytes copied.
 */
export function writeBuf(
  memory: ArrayBuffer,
  offset: number,
  maxLength: number,
  buf: Uint8Array
): number {
  const writtenLength = Math.min(maxLength, buf.length);
  const view = new Uint8Array(memory, offset, writtenLength);
  view.set(buf.subarray(0, writtenLength));
  return writtenLength;
}
