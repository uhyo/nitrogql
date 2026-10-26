// Talks to the content mapper over stdio, as TypeScript does.

import { spawn } from "node:child_process";
import { once } from "node:events";
import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { after, before, describe, it } from "node:test";
import assert from "node:assert/strict";

/**
 * Returns an absolute path in the form TypeScript uses,
 * i.e. with forward slashes even on Windows.
 */
function tsPath(relative) {
  return fileURLToPath(new URL(relative, import.meta.url)).replaceAll(
    "\\",
    "/",
  );
}

const projectDir = tsPath(".");
const tsconfig = tsPath("tsconfig.json");
const getUserFile = tsPath("src/app/getUser.graphql");
const mapperScript = fileURLToPath(
  new URL("content-mapper.mjs", import.meta.resolve("@nitrogql/cli")),
);

/**
 * Starts the content mapper and returns a client for it.
 */
function startMapper() {
  const child = spawn(process.execPath, [mapperScript], {
    cwd: projectDir,
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
    const body = Buffer.from(JSON.stringify(message), "utf8");
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
  };
}

// A broken mapper typically hangs instead of failing.
describe("content mapper protocol", { timeout: 60_000 }, () => {
  /** @type {ReturnType<typeof startMapper>} */
  let mapper;
  /** @type {string} */
  let configIdentity;
  before(() => {
    mapper = startMapper();
  });
  after(() => {
    mapper.child.kill();
  });

  it("initialize", async () => {
    const response = await mapper.request("initialize", {
      positionEncodings: ["utf-16"],
    });
    assert.deepEqual(response, {
      jsonrpc: "2.0",
      id: response.id,
      result: { positionEncoding: "utf-16", diagnosticSource: "nitrogql" },
    });
  });

  it("openProject", async () => {
    const response = await mapper.request("openProject", {
      configFileName: tsconfig,
      projectHandle: "project",
    });
    assert.equal(response.error, undefined);
    const { watchedFiles, optionDiagnostics } = response.result;
    configIdentity = response.result.configIdentity;
    assert.equal(typeof configIdentity, "string");
    assert.equal(optionDiagnostics, undefined);
    // Paths are returned in the form TypeScript sent them.
    const watched = watchedFiles.map((file) => file.slice(projectDir.length));
    for (const file of [
      "graphql.config.yaml",
      "src/schema/schema.graphql",
      "src/schema/user.graphql",
      // Operation files are watched, as adding an import to one makes it depend on another file.
      "src/app/getUser.graphql",
      "src/app/updateName.graphql",
      "src/app/userFields.graphql",
    ]) {
      assert.ok(watched.includes(file), `${file} is watched: ${watched}`);
    }
  });

  it("transform", async () => {
    const content = await readFile(getUserFile, "utf8");
    const response = await mapper.request("transform", {
      fileName: getUserFile,
      content,
      projectHandle: "project",
    });
    assert.equal(response.error, undefined);
    const { text, extension, mappings, diagnostics } = response.result;
    assert.equal(extension, ".ts");
    assert.match(text, /declare const GetUserQuery: TypedDocumentNode</);
    assert.ok(mappings.length > 0);
    assert.equal(diagnostics, undefined);
  });

  it("transform with non-ASCII content", async () => {
    // Comment shifts positions differently in bytes and UTF-16 code units
    const content =
      "# 日本語 🎉\n" +
      (await readFile(getUserFile, "utf8")).replace("age", "agee");
    const response = await mapper.request("transform", {
      fileName: getUserFile,
      content,
      projectHandle: "project",
    });
    const [diagnostic, ...rest] = response.result.diagnostics;
    assert.deepEqual(rest, []);
    // positionEncoding is utf-16, so offsets are JavaScript string offsets
    assert.equal(
      content.slice(diagnostic.start, diagnostic.start + diagnostic.length),
      "agee",
    );
  });

  it("several requests in one write", async () => {
    const content = "query { ";
    const [syntaxError, unknownProject, unknownMethod] =
      await mapper.requestAll(
        [
          "transform",
          { fileName: getUserFile, content, projectHandle: "project" },
        ],
        [
          "transform",
          { fileName: getUserFile, content, projectHandle: "unknown" },
        ],
        ["unknownMethod", {}],
      );
    assert.equal(syntaxError.result.diagnostics.length, 1);
    assert.equal(unknownProject.error.code, -32602);
    assert.equal(unknownMethod.error.code, -32601);
  });

  it("does not respond to notifications", async () => {
    mapper.notify("unknownNotification", {});
    // A request sent after the notification is answered after it would be.
    await mapper.request("unknownMethod", {});
    assert.deepEqual(mapper.unexpected, []);
  });

  it("closeProject", async () => {
    const response = await mapper.request("closeProject", {
      projectHandle: "project",
    });
    assert.equal(response.result, null);
    const transform = await mapper.request("transform", {
      fileName: getUserFile,
      content: "",
      projectHandle: "project",
    });
    assert.equal(transform.error.code, -32602);
  });

  it("reopens a closed project", async () => {
    // TypeScript closes and reopens a project when a watched file changes.
    const response = await mapper.request("openProject", {
      configFileName: tsconfig,
      projectHandle: "project",
    });
    assert.equal(response.error, undefined);
    assert.equal(response.result.configIdentity, configIdentity);
    const transform = await mapper.request("transform", {
      fileName: getUserFile,
      content: await readFile(getUserFile, "utf8"),
      projectHandle: "project",
    });
    assert.match(
      transform.result.text,
      /declare const GetUserQuery: TypedDocumentNode</,
    );
    assert.equal(transform.result.diagnostics, undefined);
  });

  it("exits when stdin is closed", async () => {
    const exit = once(mapper.child, "exit");
    mapper.child.stdin.end();
    const [code] = await exit;
    assert.equal(code, 0);
    assert.deepEqual(mapper.unexpected, []);
  });
});
