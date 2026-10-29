/**
 * @file `nitrogql_helper/content_mapper` namespace.
 *
 * Owns the stdio connection to TypeScript for nitrogql's content mapper,
 * as the wasm module cannot wait for input without blocking Node.js.
 * The wasm module receives input as is and frames messages itself;
 * only the first message is read here, to answer `initialize` early.
 */

import type { Readable, Writable } from "node:stream";
import { readBytes, writeBytes } from "./memory.js";

export type NitrogqlContentMapperNamespace = {
  /**
   * Requests the next chunk of input.
   * Result is returned asynchronously via `execute_node_ret` function.
   * Ticket fails when the input is closed.
   */
  read_input(ticket_handle: number): void;
  /**
   * Writes given bytes to the output.
   */
  write_output(data_ptr: number, data_len: number): void;
};

export type InitNitrogqlContentMapperOptions = {
  input: Readable;
  output: Writable;
  /**
   * Called when the input is closed.
   */
  onClose: () => void;
};

export type InitNitrogqlContentMapperResult = {
  namespace: NitrogqlContentMapperNamespace;
  setWasmModule: (module: WebAssembly.Exports) => void;
};

const headerDelimiter = Buffer.from("\r\n\r\n");

/**
 * Position encodings supported by the wasm module, in order of preference.
 */
const positionEncodings = ["utf-8", "utf-16"];

/**
 * Initialize the `nitrogql_helper/content_mapper` namespace.
 */
export function initContentMapperNamespace({
  input,
  output,
  onClose,
}: InitNitrogqlContentMapperOptions): InitNitrogqlContentMapperResult {
  let module: WebAssembly.Exports | undefined = undefined;
  /**
   * Received input that is not yet requested.
   */
  let chunks: Buffer[] = [];
  /**
   * Received input until the first message is complete.
   * Undefined once the first message has been handled.
   */
  let head: Buffer | undefined = Buffer.alloc(0);
  /**
   * Tickets waiting for input.
   */
  const tickets: number[] = [];
  let closed = false;

  input.on("data", (chunk: Buffer) => {
    if (head === undefined) {
      chunks.push(chunk);
    } else {
      head = Buffer.concat([head, chunk]);
      const data = handleFirstMessage(head);
      if (data === undefined) {
        return;
      }
      chunks.push(data);
      head = undefined;
    }
    flush();
  });
  input.on("end", () => {
    if (head !== undefined && head.length > 0) {
      chunks.push(head);
    }
    head = undefined;
    closed = true;
    flush();
    onClose();
  });

  return {
    namespace: {
      read_input,
      write_output,
    },
    setWasmModule: (m) => {
      module = m;
    },
  };

  function read_input(ticket_handle: number): void {
    tickets.push(ticket_handle);
    // Deliver asynchronously, as the wasm module is not reentrant.
    setImmediate(flush);
  }

  function write_output(data_ptr: number, data_len: number): void {
    // Copied, as the memory is reused after this returns.
    output.write(Buffer.from(readBytes(data_ptr, data_len)));
  }

  /**
   * Answers the first message if it is an `initialize` request.
   * Returns the input to pass to the wasm module,
   * or undefined if the first message is not complete yet.
   *
   * TypeScript gives up on a content mapper that does not answer `initialize`
   * within 5 seconds, and loading the wasm module can take seconds on a busy
   * machine. Later requests have no such limit.
   * The request is still passed to the wasm module, with only the chosen
   * position encoding, so that it uses the same encoding. It is passed as a
   * notification, without its id, so that the wasm module does not respond.
   * Messages that cannot be answered here are left to the wasm module as is,
   * which reports errors.
   */
  function handleFirstMessage(data: Buffer): Buffer | undefined {
    const headerEnd = data.indexOf(headerDelimiter);
    if (headerEnd === -1) {
      return undefined;
    }
    const header = data.subarray(0, headerEnd).toString("ascii");
    const match = /^Content-Length:\s*(\d+)\s*$/im.exec(header);
    if (match === null) {
      return data;
    }
    const messageStart = headerEnd + headerDelimiter.length;
    const messageEnd = messageStart + Number(match[1]);
    if (data.length < messageEnd) {
      return undefined;
    }
    const notification = answerInitialize(
      data.subarray(messageStart, messageEnd).toString("utf8"),
    );
    if (notification === undefined) {
      return data;
    }
    return Buffer.concat([frame(notification), data.subarray(messageEnd)]);
  }

  /**
   * Answers given message if it is an `initialize` request.
   * Returns the notification to pass to the wasm module instead,
   * or undefined if it is not answered.
   */
  function answerInitialize(message: string): string | undefined {
    let request: unknown;
    try {
      request = JSON.parse(message);
    } catch {
      return undefined;
    }
    if (
      typeof request !== "object" ||
      request === null ||
      !("method" in request) ||
      request.method !== "initialize"
    ) {
      return undefined;
    }
    const { id, ...notification } = request as {
      id?: unknown;
      params?: unknown;
    };
    const { params } = notification;
    // Notifications are not responded to, and the wasm module reports invalid params.
    if (
      id === undefined ||
      id === null ||
      typeof params !== "object" ||
      params === null ||
      !("positionEncodings" in params) ||
      !Array.isArray(params.positionEncodings)
    ) {
      return undefined;
    }
    const offered: unknown[] = params.positionEncodings;
    const encoding = positionEncodings.find((encoding) =>
      offered.includes(encoding),
    );
    if (encoding === undefined) {
      return undefined;
    }
    output.write(
      frame(
        JSON.stringify({
          jsonrpc: "2.0",
          id,
          result: { positionEncoding: encoding, diagnosticSource: "nitrogql" },
        }),
      ),
    );
    return JSON.stringify({
      ...notification,
      params: { ...params, positionEncodings: [encoding] },
    });
  }

  function flush(): void {
    if (module === undefined) {
      return;
    }
    const alloc_string = module.alloc_string as (len: number) => number;
    const free_string = module.free_string as (
      ptr: number,
      len: number,
    ) => void;
    const execute_node_ret = module.execute_node_ret as (
      ticket_handle: number,
      is_ok: number,
      result_ptr: number,
      result_len: number,
    ) => void;
    while (tickets.length > 0) {
      if (chunks.length === 0) {
        if (closed) {
          execute_node_ret(tickets.shift()!, 0, 0, 0);
          continue;
        }
        return;
      }
      // Everything received so far is delivered at once.
      const data = chunks.length === 1 ? chunks[0]! : Buffer.concat(chunks);
      chunks = [];
      const ticket_handle = tickets.shift()!;
      const data_ptr = alloc_string(data.length);
      writeBytes(data, data_ptr);
      execute_node_ret(ticket_handle, 1, data_ptr, data.length);
      free_string(data_ptr, data.length);
    }
  }
}

/**
 * Frames a message with a `Content-Length` header.
 */
function frame(message: string): Buffer {
  const body = Buffer.from(message, "utf8");
  return Buffer.concat([
    Buffer.from(`Content-Length: ${body.length}\r\n\r\n`, "ascii"),
    body,
  ]);
}
