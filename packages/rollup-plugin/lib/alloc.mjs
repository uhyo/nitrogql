export class StringAllocator {
  /**
   * 
   * @param {WebAssembly.Instance} instance 
   */
  constructor(instance) {
    this.instance = instance;
    this.encoder = new TextEncoder();
    this.decoder = new TextDecoder();
  }
  /**
   * Allocates an immutable string buffer with given content.
   */
  allocString(content) {
    const buf = this.encoder.encode(content);
    const size = buf.byteLength;
    /** @type {number} */
    const result = this.instance.exports.alloc_string(size);
    /** @type {WebAssembly.Memory} */
    const memory = this.instance.exports.memory;
    const view = new Uint8Array(memory.buffer, result, buf.byteLength);
    view.set(buf);

    let freed = false;
    return {
      ptr: result,
      size,
      free: ()=> {
        if (freed) {
          throw new Error('Double free');
        }
        freed = true;
        this.instance.exports.free_string(result, size);
      }
    }
  }

  /**
   * Reads string from given pointer and size.
   * @param {number} ptr
   * @param {number} size
   */
  readString(ptr, size) {
    const view = new Uint8Array(this.instance.exports.memory.buffer, ptr, size);
    const decoded = this.decoder.decode(view);
    return decoded;
  }
}