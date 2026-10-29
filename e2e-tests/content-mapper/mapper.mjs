// Client of the content mapper, which talks to it over stdio as TypeScript does.

import { spawn } from "node:child_process";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

/**
 * Returns an absolute path in the form TypeScript uses,
 * i.e. with forward slashes even on Windows.
 */
export function tsPath(url) {
  return fileURLToPath(url).replaceAll("\\", "/");
}

const mapperScript = fileURLToPath(
  new URL("content-mapper.mjs", import.meta.resolve("@nitrogql/cli")),
);

/**
 * Path to a native build of nitrogql, which is tested instead of the wasm build if given.
 * A relative path is resolved from the current directory.
 */
export const nativeBinary =
  process.env.NITROGQL_NATIVE_BINARY &&
  resolve(process.env.NITROGQL_NATIVE_BINARY);

/**
 * Starts the content mapper in given directory and returns a client for it.
 * `execArgv` is passed to Node.js, and is not available for the native build.
 */
export function startMapper(cwd, execArgv = []) {
  const [command, args] = nativeBinary
    ? [nativeBinary, ["content-mapper"]]
    : [process.execPath, [...execArgv, mapperScript]];
  const child = spawn(command, args, {
    cwd,
    stdio: ["pipe", "pipe", "inherit"],
  });
  /** @type {Map<number, (response: any) => void>} */
  const pending = new Map();
  /** @type {any[]} */
  const unexpected = [];
  let nextId = 1;
  let buffer = Buffer.alloc(0);
  child.stdout.on("data", (chunk) => {
    buffer = Buffer.concat([buffer, chunk]);
    while (true) {
      const headerEnd = buffer.indexOf("\r\n\r\n");
      if (headerEnd === -1) {
        return;
      }
      const header = buffer.subarray(0, headerEnd).toString("ascii");
      const length = Number(/^Content-Length: (\d+)$/m.exec(header)?.[1]);
      const start = headerEnd + 4;
      if (buffer.length < start + length) {
        return;
      }
      const response = JSON.parse(
        buffer.subarray(start, start + length).toString("utf8"),
      );
      buffer = buffer.subarray(start + length);
      const resolve = pending.get(response.id);
      if (resolve === undefined) {
        unexpected.push(response);
      } else {
        pending.delete(response.id);
        resolve(response);
      }
    }
  });

  function frame(message) {
    return frameText(JSON.stringify(message));
  }

  function frameText(text) {
    const body = Buffer.from(text, "utf8");
    return Buffer.concat([
      Buffer.from(`Content-Length: ${body.length}\r\n\r\n`, "ascii"),
      body,
    ]);
  }

  return {
    child,
    unexpected,
    /**
     * Sends requests in one write and waits for all responses.
     */
    requestAll(...requests) {
      const messages = requests.map(([method, params]) => ({
        jsonrpc: "2.0",
        id: nextId++,
        method,
        params,
      }));
      const responses = messages.map(
        (message) => new Promise((resolve) => pending.set(message.id, resolve)),
      );
      child.stdin.write(Buffer.concat(messages.map(frame)));
      return Promise.all(responses);
    },
    request(method, params) {
      return this.requestAll([method, params]).then(([response]) => response);
    },
    notify(method, params) {
      child.stdin.write(frame({ jsonrpc: "2.0", method, params }));
    },
    /**
     * Sends given text as a message and waits for the response with given id.
     */
    sendRaw(text, id) {
      const response = new Promise((resolve) => pending.set(id, resolve));
      child.stdin.write(frameText(text));
      return response;
    },
  };
}
