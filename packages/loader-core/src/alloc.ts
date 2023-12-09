import { WASMInstance } from "./wasm.js";

export type AllocatedString = {
  ptr: number;
  size: number;
  free: () => void;
};

export class StringAllocator {
  readonly instance: WASMInstance;
  readonly encoder: TextEncoder;
  readonly decoder: TextDecoder;
  constructor(instance: WASMInstance) {
    this.instance = instance;
    this.encoder = new TextEncoder();
    this.decoder = new TextDecoder();
  }
  /**
   * Allocates an immutable string buffer with given content.
   */
  allocString(content: string): AllocatedString {
    const buf = this.encoder.encode(content);
    const size = buf.byteLength;
    const result = this.instance.exports.alloc_string(size);
    const memory = this.instance.exports.memory;
    const view = new Uint8Array(memory.buffer, result, buf.byteLength);
    view.set(buf);

    let freed = false;
    return {
      ptr: result,
      size,
      free: () => {
        if (freed) {
          throw new Error("Double free");
        }
        freed = true;
        this.instance.exports.free_string(result, size);
      },
    };
  }

  /**
   * Reads string from given pointer and size.
   */
  readString(ptr: number, size: number) {
    const view = new Uint8Array(this.instance.exports.memory.buffer, ptr, size);
    const decoded = this.decoder.decode(view);
    return decoded;
  }
}
