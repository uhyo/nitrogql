import { PassThrough } from "node:stream";
import { beforeEach, describe, expect, it, vi, type Mock } from "vitest";
import { initContentMapperNamespace } from "../contentMapper.js";
import { readString, setMemory, utf8Len, writeString } from "../memory.js";

type Delivery =
  | { ticket: number; ok: true; message: string }
  | { ticket: number; ok: false };

let input: PassThrough;
let output: PassThrough;
let onClose: Mock<() => void>;
let namespace: ReturnType<typeof initContentMapperNamespace>["namespace"];
/**
 * Results delivered to the fake wasm module via `execute_node_ret`.
 */
let deliveries: Delivery[];
/**
 * Number of strings allocated by `alloc_string` and not yet freed.
 */
let liveStrings: number;

beforeEach(() => {
  // 1 is 64KiB
  const memory = new WebAssembly.Memory({ initial: 1 });
  setMemory(memory);
  input = new PassThrough();
  output = new PassThrough();
  onClose = vi.fn();
  deliveries = [];
  liveStrings = 0;
  let nextPtr = 0;
  const fakeModule = {
    alloc_string(len: number) {
      liveStrings++;
      const ptr = nextPtr;
      nextPtr += len;
      return ptr;
    },
    free_string() {
      liveStrings--;
    },
    execute_node_ret(
      ticket: number,
      isOk: number,
      resultPtr: number,
      resultLen: number,
    ) {
      deliveries.push(
        isOk
          ? { ticket, ok: true, message: readString(resultPtr, resultLen) }
          : { ticket, ok: false },
      );
    },
  };
  const helper = initContentMapperNamespace({ input, output, onClose });
  helper.setWasmModule(fakeModule as unknown as WebAssembly.Exports);
  namespace = helper.namespace;
});

/**
 * Lets pending stream events and `setImmediate` callbacks run.
 */
async function settle(): Promise<void> {
  for (let i = 0; i < 3; i++) {
    await new Promise((resolve) => setImmediate(resolve));
  }
}

function frame(message: string): Buffer {
  const body = Buffer.from(message, "utf8");
  return Buffer.concat([
    Buffer.from(`Content-Length: ${body.length}\r\n\r\n`, "ascii"),
    body,
  ]);
}

function received(...messages: [number, string][]): Delivery[] {
  return messages.map(([ticket, message]) => ({ ticket, ok: true, message }));
}

describe("reading messages", () => {
  it("delivers a message requested before it arrives", async () => {
    namespace.read_message(1);
    await settle();
    expect(deliveries).toEqual([]);
    input.write(frame('{"id":1}'));
    await settle();
    expect(deliveries).toEqual(received([1, '{"id":1}']));
  });

  it("queues messages that arrive before they are requested", async () => {
    input.write(frame('{"id":1}'));
    input.write(frame('{"id":2}'));
    await settle();
    expect(deliveries).toEqual([]);
    namespace.read_message(10);
    namespace.read_message(11);
    await settle();
    expect(deliveries).toEqual(received([10, '{"id":1}'], [11, '{"id":2}']));
  });

  it("delivers messages asynchronously", async () => {
    input.write(frame('{"id":1}'));
    await settle();
    namespace.read_message(1);
    // The wasm module is not reentrant, so it must not be called back
    // from within read_message.
    expect(deliveries).toEqual([]);
    await settle();
    expect(deliveries).toEqual(received([1, '{"id":1}']));
  });

  it("delivers one message per request", async () => {
    input.write(Buffer.concat([frame('{"id":1}'), frame('{"id":2}')]));
    namespace.read_message(1);
    await settle();
    expect(deliveries).toEqual(received([1, '{"id":1}']));
    namespace.read_message(2);
    await settle();
    expect(deliveries).toEqual(received([1, '{"id":1}'], [2, '{"id":2}']));
  });

  it("frees strings after delivering them", async () => {
    input.write(Buffer.concat([frame('{"id":1}'), frame('{"id":2}')]));
    namespace.read_message(1);
    namespace.read_message(2);
    await settle();
    expect(deliveries).toHaveLength(2);
    expect(liveStrings).toBe(0);
  });
});

describe("framing", () => {
  it("handles a message split across several chunks", async () => {
    namespace.read_message(1);
    const data = frame('{"jsonrpc":"2.0","id":1,"method":"initialize"}');
    // split inside the header, at the header delimiter, and inside the body
    for (const [start, end] of [
      [0, 5],
      [5, 18],
      [18, 21],
      [21, 30],
      [30, data.length],
    ] as const) {
      input.write(data.subarray(start, end));
      await settle();
    }
    expect(deliveries).toEqual(
      received([1, '{"jsonrpc":"2.0","id":1,"method":"initialize"}']),
    );
  });

  it("concatenates a large message once after all chunks arrive", async () => {
    namespace.read_message(1);
    const message = JSON.stringify({ id: 1, text: "x".repeat(10000) });
    const data = frame(message);
    const concat = vi.spyOn(Buffer, "concat");
    try {
      for (let start = 0; start < data.length; start += 100) {
        input.write(data.subarray(start, start + 100));
        await settle();
      }
      expect(deliveries).toEqual(received([1, message]));
      // once until the header is complete, and once for the whole message
      expect(concat.mock.calls.length).toBeLessThanOrEqual(2);
    } finally {
      concat.mockRestore();
    }
  });

  it("handles several messages in one chunk", async () => {
    namespace.read_message(1);
    namespace.read_message(2);
    namespace.read_message(3);
    input.write(
      Buffer.concat([frame('{"id":1}'), frame('{"id":2}'), frame('{"id":3}')]),
    );
    await settle();
    expect(deliveries).toEqual(
      received([1, '{"id":1}'], [2, '{"id":2}'], [3, '{"id":3}']),
    );
  });

  it("handles a chunk that ends in the middle of the next message", async () => {
    namespace.read_message(1);
    namespace.read_message(2);
    const data = Buffer.concat([frame('{"id":1}'), frame('{"id":2}')]);
    const splitAt = frame('{"id":1}').length + 3;
    input.write(data.subarray(0, splitAt));
    await settle();
    expect(deliveries).toEqual(received([1, '{"id":1}']));
    input.write(data.subarray(splitAt));
    await settle();
    expect(deliveries).toEqual(received([1, '{"id":1}'], [2, '{"id":2}']));
  });

  it("accepts a header with extra fields", async () => {
    namespace.read_message(1);
    namespace.read_message(2);
    const body = '{"id":1}';
    input.write(
      `Content-Type: application/vscode-jsonrpc; charset=utf-8\r\nContent-Length: ${body.length}\r\n\r\n${body}`,
    );
    input.write(
      `Content-Length: ${body.length}\r\nContent-Type: application/vscode-jsonrpc; charset=utf-8\r\n\r\n${body}`,
    );
    await settle();
    expect(deliveries).toEqual(received([1, body], [2, body]));
  });

  it("accepts a header field name in any case", async () => {
    namespace.read_message(1);
    input.write('content-length: 8\r\n\r\n{"id":1}');
    await settle();
    expect(deliveries).toEqual(received([1, '{"id":1}']));
  });

  it("skips a message without Content-Length", async () => {
    const consoleError = vi
      .spyOn(console, "error")
      .mockImplementation(() => {});
    try {
      namespace.read_message(1);
      input.write("Content-Type: application/json\r\n\r\n");
      input.write(frame('{"id":1}'));
      await settle();
      expect(deliveries).toEqual(received([1, '{"id":1}']));
      expect(consoleError).toHaveBeenCalledOnce();
    } finally {
      consoleError.mockRestore();
    }
  });

  it("uses byte length for non-ASCII content", async () => {
    namespace.read_message(1);
    namespace.read_message(2);
    const message = '{"text":"日本語 🎉"}';
    expect(Buffer.byteLength(message)).not.toBe(message.length);
    const data = Buffer.concat([frame(message), frame('{"id":2}')]);
    // split inside a multi-byte character
    const splitAt = data.indexOf(Buffer.from("本")) + 1;
    input.write(data.subarray(0, splitAt));
    await settle();
    expect(deliveries).toEqual([]);
    input.write(data.subarray(splitAt));
    await settle();
    expect(deliveries).toEqual(received([1, message], [2, '{"id":2}']));
  });
});

describe("closing", () => {
  it("fails a pending read when input is closed", async () => {
    namespace.read_message(1);
    await settle();
    input.end();
    await settle();
    expect(deliveries).toEqual([{ ticket: 1, ok: false }]);
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("delivers received messages before failing reads", async () => {
    namespace.read_message(1);
    namespace.read_message(2);
    input.end(frame('{"id":1}'));
    await settle();
    expect(deliveries).toEqual([
      { ticket: 1, ok: true, message: '{"id":1}' },
      { ticket: 2, ok: false },
    ]);
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("fails reads requested after input is closed", async () => {
    input.end();
    await settle();
    expect(onClose).toHaveBeenCalledOnce();
    namespace.read_message(1);
    await settle();
    expect(deliveries).toEqual([{ ticket: 1, ok: false }]);
  });

  it("does not call onClose while input is open", async () => {
    input.write(frame('{"id":1}'));
    await settle();
    expect(onClose).not.toHaveBeenCalled();
  });
});

describe("writing messages", () => {
  function writeMessage(message: string): void {
    const len = utf8Len(message);
    // Somewhere not used by the fake allocator in these tests
    const ptr = 32768;
    writeString(message, ptr, len);
    namespace.write_message(ptr, len);
  }

  it("frames a message with Content-Length", () => {
    writeMessage('{"jsonrpc":"2.0","id":1,"result":null}');
    expect(output.read().toString("utf8")).toBe(
      'Content-Length: 38\r\n\r\n{"jsonrpc":"2.0","id":1,"result":null}',
    );
  });

  it("uses byte length for non-ASCII content", () => {
    const message = '{"text":"日本語 🎉"}';
    writeMessage(message);
    const data: Buffer = output.read();
    expect(data.equals(frame(message))).toBe(true);
  });
});

describe("initialize", () => {
  const initialize = (positionEncodings: string[]) =>
    JSON.stringify({
      jsonrpc: "2.0",
      id: 1,
      method: "initialize",
      params: { positionEncodings },
    });

  function writeMessage(message: string): void {
    const len = utf8Len(message);
    const ptr = 32768;
    writeString(message, ptr, len);
    namespace.write_message(ptr, len);
  }

  /**
   * Returns everything written to the output since the last call.
   */
  function sent(): string {
    // Depending on the Node.js version, one read may return only one chunk.
    let result = "";
    let chunk: Buffer | null;
    while ((chunk = output.read()) !== null) {
      result += chunk.toString("utf8");
    }
    return result;
  }

  it("answers before the wasm module is ready", async () => {
    // Without the wasm module, unlike the one created in beforeEach
    const input = new PassThrough();
    const output = new PassThrough();
    initContentMapperNamespace({ input, output, onClose });
    input.write(frame(initialize(["utf-16"])));
    await settle();
    expect(output.read().toString("utf8")).toBe(
      frame(
        '{"jsonrpc":"2.0","id":1,"result":{"positionEncoding":"utf-16","diagnosticSource":"nitrogql"}}',
      ).toString("utf8"),
    );
  });

  it("passes the answered request as a notification with the chosen encoding only", async () => {
    namespace.read_message(1);
    input.write(frame(initialize(["utf-16", "utf-8"])));
    await settle();
    expect(sent()).toContain('"positionEncoding":"utf-8"');
    // Without an id, so that the wasm module does not respond.
    expect(deliveries).toEqual(
      received([
        1,
        '{"jsonrpc":"2.0","method":"initialize","params":{"positionEncodings":["utf-8"]}}',
      ]),
    );
  });

  it("sends every message of the wasm module", async () => {
    input.write(frame(initialize(["utf-16"])));
    await settle();
    sent();
    writeMessage('{"jsonrpc":"2.0","id":1,"result":null}');
    expect(sent()).toBe(
      frame('{"jsonrpc":"2.0","id":1,"result":null}').toString("utf8"),
    );
  });

  it("leaves a request without a supported encoding to the wasm module", async () => {
    namespace.read_message(1);
    input.write(frame(initialize(["utf-32"])));
    await settle();
    expect(sent()).toBe("");
    expect(deliveries).toEqual(received([1, initialize(["utf-32"])]));
  });

  it("leaves a notification to the wasm module", async () => {
    namespace.read_message(1);
    const notification =
      '{"jsonrpc":"2.0","method":"initialize","params":{"positionEncodings":["utf-8"]}}';
    input.write(frame(notification));
    await settle();
    expect(sent()).toBe("");
    expect(deliveries).toEqual(received([1, notification]));
  });

  it("only answers the first initialize request", async () => {
    namespace.read_message(1);
    namespace.read_message(2);
    input.write(frame(initialize(["utf-32"])));
    input.write(frame(initialize(["utf-16"])));
    await settle();
    expect(sent()).toBe("");
    expect(deliveries).toEqual(
      received([1, initialize(["utf-32"])], [2, initialize(["utf-16"])]),
    );
  });
});
