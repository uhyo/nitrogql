// Talks to the content mapper over stdio, as TypeScript does.

import { once } from "node:events";
import { readFile } from "node:fs/promises";
import { after, before, describe, it } from "node:test";
import assert from "node:assert/strict";

import { startMapper, tsPath as toTsPath } from "./mapper.mjs";

function tsPath(relative) {
  return toTsPath(new URL(relative, import.meta.url));
}

const projectDir = tsPath(".");
const tsconfig = tsPath("tsconfig.json");
const getUserFile = tsPath("src/app/getUser.graphql");
// A broken mapper typically hangs instead of failing.
describe("content mapper protocol", { timeout: 60_000 }, () => {
  /** @type {ReturnType<typeof startMapper>} */
  let mapper;
  /** @type {string} */
  let configIdentity;
  before(() => {
    mapper = startMapper(projectDir);
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
    ]) {
      assert.ok(watched.includes(file), `${file} is watched: ${watched}`);
    }
    // Operation files are not watched; TypeScript tracks imports between them.
    assert.ok(
      !watched.includes("src/app/userFields.graphql"),
      `operation files are not watched: ${watched}`,
    );
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

  it("responds to messages that are not JSON", async () => {
    const response = await mapper.sendRaw("{", null);
    assert.equal(response.error.code, -32700);
  });

  it("responds to invalid requests", async () => {
    const response = await mapper.sendRaw(
      JSON.stringify({ jsonrpc: "2.0", id: "invalid", method: 1 }),
      "invalid",
    );
    assert.equal(response.error.code, -32600);
    // Requests after invalid ones are still handled.
    const next = await mapper.request("unknownMethod", {});
    assert.equal(next.error.code, -32601);
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

describe("content mapper startup", { timeout: 60_000 }, () => {
  it("answers initialize before the wasm module is compiled", async () => {
    // TypeScript gives up on a content mapper that takes more than 5 seconds
    // to answer initialize, while compiling the wasm module can be slow.
    const neverCompile = "WebAssembly.compile = () => new Promise(() => {});";
    const mapper = startMapper(projectDir, [
      "--import",
      `data:text/javascript,${encodeURIComponent(neverCompile)}`,
    ]);
    try {
      const response = await mapper.request("initialize", {
        positionEncodings: ["utf-16", "utf-8"],
      });
      assert.deepEqual(response, {
        jsonrpc: "2.0",
        id: response.id,
        result: { positionEncoding: "utf-8", diagnosticSource: "nitrogql" },
      });
    } finally {
      mapper.child.kill();
    }
  });
});
