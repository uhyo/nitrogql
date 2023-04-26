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

/**
 * Reads string from memory.
 */
export function readString(ptr: number, len: number): string {
  const memory = getMemory();
  const bytes = new Uint8Array(memory.buffer, ptr, len);
  return utf8decoder.decode(bytes);
}
