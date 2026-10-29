import { PassThrough } from "node:stream";
import { beforeEach, describe, expect, it, vi, type Mock } from "vitest";
import { initContentMapperNamespace } from "../contentMapper.js";
import { readBytes, setMemory, writeBytes } from "../memory.js";

type Delivery =
  | { ticket: number; ok: true; data: Buffer }
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
          ? {
              ticket,
              ok: true,
              data: Buffer.from(readBytes(resultPtr, resultLen)),
            }
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

function received(...deliveries: [number, Buffer | string][]): Delivery[] {
  return deliveries.map(([ticket, data]) => ({
    ticket,
    ok: true,
    data: Buffer.from(data),
  }));
}

const initialize = (positionEncodings: string[]) =>
  JSON.stringify({
    jsonrpc: "2.0",
    id: 1,
    method: "initialize",
    params: { positionEncodings },
  });

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

describe("reading input", () => {
  it("delivers input requested before it arrives", async () => {
    namespace.read_input(1);
    await settle();
    expect(deliveries).toEqual([]);
    input.write(frame('{"id":1}'));
    await settle();
    expect(deliveries).toEqual(received([1, frame('{"id":1}')]));
  });

  it("delivers everything received before a request at once", async () => {
    input.write(frame('{"id":1}'));
    input.write(frame('{"id":2}'));
    input.write(frame('{"id":3}'));
    await settle();
    expect(deliveries).toEqual([]);
    namespace.read_input(10);
    namespace.read_input(11);
    await settle();
    expect(deliveries).toEqual(
      received([
        10,
        Buffer.concat([
          frame('{"id":1}'),
          frame('{"id":2}'),
          frame('{"id":3}'),
        ]),
      ]),
    );
    input.write(frame('{"id":4}'));
    await settle();
    expect(deliveries.slice(1)).toEqual(received([11, frame('{"id":4}')]));
  });

  it("delivers input asynchronously", async () => {
    input.write(frame('{"id":1}'));
    await settle();
    namespace.read_input(1);
    // The wasm module is not reentrant, so it must not be called back
    // from within read_input.
    expect(deliveries).toEqual([]);
    await settle();
    expect(deliveries).toEqual(received([1, frame('{"id":1}')]));
  });

  it("frees memory after delivering input", async () => {
    input.write(frame('{"id":1}'));
    namespace.read_input(1);
    await settle();
    input.write(frame('{"id":2}'));
    namespace.read_input(2);
    await settle();
    expect(deliveries).toHaveLength(2);
    expect(liveStrings).toBe(0);
  });

  it("delivers input as is after the first message", async () => {
    // Framing is left to the wasm module.
    input.write(frame('{"id":1}'));
    namespace.read_input(1);
    await settle();
    const data = frame('{"text":"日本語"}');
    // split inside a multi-byte character, and without a complete header
    const splitAt = data.indexOf(Buffer.from("本")) + 1;
    input.write(data.subarray(0, splitAt));
    namespace.read_input(2);
    await settle();
    input.write(data.subarray(splitAt));
    namespace.read_input(3);
    await settle();
    expect(deliveries).toEqual(
      received(
        [1, frame('{"id":1}')],
        [2, data.subarray(0, splitAt)],
        [3, data.subarray(splitAt)],
      ),
    );
  });
});

describe("first message", () => {
  it("waits until the first message is complete", async () => {
    namespace.read_input(1);
    const data = Buffer.concat([frame('{"id":1}'), frame('{"id":2}')]);
    // split inside the header, at the header delimiter, and inside the body
    for (const [start, end] of [
      [0, 5],
      [5, 18],
      [18, 21],
    ] as const) {
      input.write(data.subarray(start, end));
      await settle();
      expect(deliveries).toEqual([]);
    }
    input.write(data.subarray(21));
    await settle();
    expect(deliveries).toEqual(received([1, data]));
  });

  it("passes a first message without Content-Length as is", async () => {
    namespace.read_input(1);
    const data = Buffer.from("Content-Type: application/json\r\n\r\n{}");
    input.write(data);
    await settle();
    expect(deliveries).toEqual(received([1, data]));
  });

  it("answers initialize before the wasm module is ready", async () => {
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
    namespace.read_input(1);
    input.write(
      Buffer.concat([
        frame(initialize(["utf-16", "utf-8"])),
        frame('{"id":2}'),
      ]),
    );
    await settle();
    expect(sent()).toContain('"positionEncoding":"utf-8"');
    // Without an id, so that the wasm module does not respond.
    expect(deliveries).toEqual(
      received([
        1,
        Buffer.concat([
          frame(
            '{"jsonrpc":"2.0","method":"initialize","params":{"positionEncodings":["utf-8"]}}',
          ),
          frame('{"id":2}'),
        ]),
      ]),
    );
  });

  it("leaves a request without a supported encoding to the wasm module", async () => {
    namespace.read_input(1);
    input.write(frame(initialize(["utf-32"])));
    await settle();
    expect(sent()).toBe("");
    expect(deliveries).toEqual(received([1, frame(initialize(["utf-32"]))]));
  });

  it("leaves a notification to the wasm module", async () => {
    namespace.read_input(1);
    const notification =
      '{"jsonrpc":"2.0","method":"initialize","params":{"positionEncodings":["utf-8"]}}';
    input.write(frame(notification));
    await settle();
    expect(sent()).toBe("");
    expect(deliveries).toEqual(received([1, frame(notification)]));
  });

  it("only answers the first message", async () => {
    namespace.read_input(1);
    const data = Buffer.concat([
      frame('{"jsonrpc":"2.0","method":"initialized"}'),
      frame(initialize(["utf-16"])),
    ]);
    input.write(data);
    await settle();
    expect(sent()).toBe("");
    expect(deliveries).toEqual(received([1, data]));
  });
});

describe("closing", () => {
  it("fails a pending read when input is closed", async () => {
    namespace.read_input(1);
    await settle();
    input.end();
    await settle();
    expect(deliveries).toEqual([{ ticket: 1, ok: false }]);
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("delivers received input before failing reads", async () => {
    namespace.read_input(1);
    namespace.read_input(2);
    input.end(frame('{"id":1}'));
    await settle();
    expect(deliveries).toEqual([
      ...received([1, frame('{"id":1}')]),
      { ticket: 2, ok: false },
    ]);
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("delivers an incomplete first message when input is closed", async () => {
    namespace.read_input(1);
    namespace.read_input(2);
    input.end("Content-Length: 10\r\n");
    await settle();
    expect(deliveries).toEqual([
      ...received([1, "Content-Length: 10\r\n"]),
      { ticket: 2, ok: false },
    ]);
  });

  it("fails reads requested after input is closed", async () => {
    input.end();
    await settle();
    expect(onClose).toHaveBeenCalledOnce();
    namespace.read_input(1);
    await settle();
    expect(deliveries).toEqual([{ ticket: 1, ok: false }]);
  });

  it("does not call onClose while input is open", async () => {
    input.write(frame('{"id":1}'));
    await settle();
    expect(onClose).not.toHaveBeenCalled();
  });
});

describe("writing output", () => {
  it("writes bytes as is", () => {
    const data = frame('{"text":"日本語 🎉"}');
    // Somewhere not used by the fake allocator in these tests
    const ptr = 32768;
    writeBytes(data, ptr);
    namespace.write_output(ptr, data.length);
    // Overwriting the memory does not change what was written.
    writeBytes(Buffer.alloc(data.length), ptr);
    expect(Buffer.from(sent(), "utf8").equals(data)).toBe(true);
  });
});
