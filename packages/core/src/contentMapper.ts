/**
 * @file `nitrogql_helper/content_mapper` namespace.
 *
 * Handles the stdio connection to TypeScript for nitrogql's content mapper.
 * Messages are framed with `Content-Length` headers, as in the Language Server Protocol.
 */

import type { Readable, Writable } from "node:stream";
import { readString, utf8Len, writeString } from "./memory.js";

export type NitrogqlContentMapperNamespace = {
  /**
   * Requests the next message.
   * Result is returned asynchronously via `execute_node_ret` function.
   * Ticket fails when the connection is closed.
   */
  read_message(ticket_handle: number): void;
  /**
   * Sends a message.
   */
  write_message(message_ptr: number, message_len: number): void;
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
   * Received messages that are not yet requested.
   */
  const messages: string[] = [];
  /**
   * Tickets waiting for a message.
   */
  const tickets: number[] = [];
  let buffer = Buffer.alloc(0);
  let closed = false;
  /**
   * Whether an `initialize` request has been received.
   */
  let initializeReceived = false;
  /**
   * JSON of ids of requests answered by this host.
   * Responses of the wasm module to them are not sent.
   */
  const answeredIds = new Set<string>();

  input.on("data", (chunk: Buffer) => {
    buffer = Buffer.concat([buffer, chunk]);
    while (true) {
      const headerEnd = buffer.indexOf(headerDelimiter);
      if (headerEnd === -1) {
        break;
      }
      const header = buffer.subarray(0, headerEnd).toString("ascii");
      const match = /^Content-Length:\s*(\d+)\s*$/im.exec(header);
      if (match === null) {
        console.error("Missing Content-Length header");
        buffer = buffer.subarray(headerEnd + headerDelimiter.length);
        continue;
      }
      const messageStart = headerEnd + headerDelimiter.length;
      const messageEnd = messageStart + Number(match[1]);
      if (buffer.length < messageEnd) {
        break;
      }
      const message = buffer
        .subarray(messageStart, messageEnd)
        .toString("utf8");
      messages.push(initializeReceived ? message : answerInitialize(message));
      buffer = buffer.subarray(messageEnd);
    }
    flush();
  });
  input.on("end", () => {
    closed = true;
    flush();
    onClose();
  });

  return {
    namespace: {
      read_message,
      write_message,
    },
    setWasmModule: (m) => {
      module = m;
    },
  };

  function read_message(ticket_handle: number): void {
    tickets.push(ticket_handle);
    // Deliver asynchronously, as the wasm module is not reentrant.
    setImmediate(flush);
  }

  function write_message(message_ptr: number, message_len: number): void {
    const message = readString(message_ptr, message_len);
    if (answeredIds.size > 0) {
      const id = JSON.stringify(JSON.parse(message).id);
      if (answeredIds.delete(id)) {
        return;
      }
    }
    send(message);
  }

  function send(message: string): void {
    output.write(
      `Content-Length: ${Buffer.byteLength(message, "utf8")}\r\n\r\n${message}`,
    );
  }

  /**
   * Answers an `initialize` request without waiting for the wasm module.
   * Returns the message to pass to the wasm module.
   *
   * TypeScript gives up on a content mapper that does not answer `initialize`
   * within 5 seconds, and loading the wasm module can take seconds on a busy
   * machine. Later requests have no such limit.
   * The request is still passed to the wasm module, with only the chosen
   * position encoding, so that it uses the same encoding.
   * Requests that cannot be answered here are left to the wasm module,
   * which reports errors.
   */
  function answerInitialize(message: string): string {
    let request: unknown;
    try {
      request = JSON.parse(message);
    } catch {
      return message;
    }
    if (
      typeof request !== "object" ||
      request === null ||
      !("method" in request) ||
      request.method !== "initialize"
    ) {
      return message;
    }
    initializeReceived = true;
    const { id, params } = request as { id?: unknown; params?: unknown };
    // Notifications are not responded to, and the wasm module reports invalid params.
    if (
      id === undefined ||
      id === null ||
      typeof params !== "object" ||
      params === null ||
      !("positionEncodings" in params) ||
      !Array.isArray(params.positionEncodings)
    ) {
      return message;
    }
    const offered: unknown[] = params.positionEncodings;
    const encoding = positionEncodings.find((encoding) =>
      offered.includes(encoding),
    );
    if (encoding === undefined) {
      return message;
    }
    send(
      JSON.stringify({
        jsonrpc: "2.0",
        id,
        result: { positionEncoding: encoding, diagnosticSource: "nitrogql" },
      }),
    );
    answeredIds.add(JSON.stringify(id));
    return JSON.stringify({
      ...request,
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
      const message = messages.shift();
      if (message === undefined) {
        if (closed) {
          execute_node_ret(tickets.shift()!, 0, 0, 0);
          continue;
        }
        return;
      }
      const ticket_handle = tickets.shift()!;
      const len = utf8Len(message);
      const message_ptr = alloc_string(len);
      writeString(message, message_ptr, len);
      execute_node_ret(ticket_handle, 1, message_ptr, len);
      free_string(message_ptr, len);
    }
  }
}
