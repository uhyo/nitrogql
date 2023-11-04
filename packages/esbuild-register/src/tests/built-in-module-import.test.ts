import { expect, it, describe } from "vitest";
import { tmp } from "./fs.js";
import { runNode } from "./runFile.js";

describe("import files locally", async () => {
  it(".mts -> builtin (without node: prefix)", async () => {
    const filePath = await tmp()
      .file(
        "entry.mts",
        `
import { posix } from "path";
console.log(posix.join("/foo/bar", "baz"));
`
      )
      .path("entry.mts");
    const result = await runNode(filePath);
    expect(result).toBe("/foo/bar/baz\n");
  });
  it(".mts -> builtin (with node: prefix)", async () => {
    const filePath = await tmp()
      .file(
        "entry.mts",
        `
import path from "node:path/posix";
console.log(path.join("/foo/bar", "baz"));
`
      )
      .path("entry.mts");
    const result = await runNode(filePath);
    expect(result).toBe("/foo/bar/baz\n");
  });
  it(".mts -> .cts -> builtin (without node: prefix)", async () => {
    const filePath = await tmp()
      .file(
        "mid.cts",
        `
import { posix } from "path";
export const joined = posix.join("/foo/bar", "../baz");
`
      )
      .file(
        "entry.mts",
        `
import mod from "./mid.cjs";
console.log(mod.joined);
`
      )
      .path("entry.mts");
    const result = await runNode(filePath);
    expect(result).toBe("/foo/baz\n");
  });
  it(".mts -> .cts -> builtin (with node: prefix)", async () => {
    const filePath = await tmp()
      .file(
        "mid.cts",
        `
import path from "node:path/posix";
export const joined = path.join("/foo/bar", "../baz");
`
      )
      .file(
        "entry.mts",
        `
import mod from "./mid.cjs";
console.log(mod.joined);
`
      )
      .path("entry.mts");
    const result = await runNode(filePath);
    expect(result).toBe("/foo/baz\n");
  });
});
