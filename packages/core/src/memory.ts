/**
 * Reference to the memory used by running wasm module.
 */
let memory: WebAssembly.Memory | undefined;

export function getMemory(): WebAssembly.Memory {
  if (memory === undefined) {
    throw new Error("Memory is not initialized");
  }
  return memory;
}

export function setMemory(newMemory: WebAssembly.Memory): void {
  memory = newMemory;
}

const utf8decoder = new TextDecoder("utf-8");
const utf8encoder = new TextEncoder();

/**
 * Reads string from memory.
 */
export function readString(ptr: number, len: number): string {
  const memory = getMemory();
  const bytes = new Uint8Array(memory.buffer, ptr, len);
  return utf8decoder.decode(bytes);
}

/**
 * Calculates the UTF-8 length of given string.
 */
export function utf8Len(str: string): number {
  return utf8encoder.encode(str).length;
}

/**
 * Writes string to memory.
 */
export function writeString(str: string, ptr: number, len: number): void {
  const memory = getMemory();
  const bytes = utf8encoder.encode(str);
  const bytesToWrite = Math.min(bytes.length, len);
  const bufferView = new Uint8Array(memory.buffer, ptr, bytesToWrite);
  bufferView.set(bytes.slice(0, bytesToWrite));
}
