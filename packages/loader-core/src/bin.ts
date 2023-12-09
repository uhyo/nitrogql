import { StringAllocator } from "./alloc.js";
import { WASMInstance, WASMInterface } from "./wasm.js";

export class WASMBin {
  readonly exports: WASMInterface;
  readonly alloc: StringAllocator;
  constructor(instance: WASMInstance) {
    this.exports = instance.exports;
    this.alloc = new StringAllocator(instance);
  }

  /**
   * Reads the result buffer.
   */
  readResult() {
    const ptr = this.exports.get_result_ptr();
    const size = this.exports.get_result_size();
    const result = this.alloc.readString(ptr, size);
    return result;
  }
}

export class WasmError extends Error {
  constructor(message: string, bin: WASMBin) {
    const errorMessage = bin.readResult();
    super(`${message}\n${errorMessage}`);
    this.name = "WasmError";
  }
}
