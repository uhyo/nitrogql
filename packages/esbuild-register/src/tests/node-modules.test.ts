import { expect, it, describe } from "vitest";
import { Folder, tmp } from "./fs.js";
import { runNode } from "./runFile.js";

const addESMPackage = (folder: Folder): Folder => {
  return folder
    .dir("node_modules")
    .dir("node_modules/foo")
    .file(
      "node_modules/foo/package.json",
      JSON.stringify({
        name: "foo",
        main: "index.js",
        type: "module",
      })
    )
    .file(
      "node_modules/foo/prefix.js",
      `
export const prefix = "This is ";
`
    )
    .file(
      "node_modules/foo/index.js",
      `
import { prefix } from "./prefix.js";
export const foo = prefix + "foo";
`
    )
    .file(
      "node_modules/foo/index.ts",
      `
throw new Error("This should not be loaded");
`
    );
};

const addCJSPackage = (folder: Folder): Folder => {
  return folder
    .dir("node_modules")
    .dir("node_modules/bar")
    .file(
      "node_modules/bar/package.json",
      JSON.stringify({
        name: "bar",
        main: "index.js",
      })
    )
    .file(
      "node_modules/bar/prefix.js",
      `
exports.prefix = "This is ";
`
    )
    .file(
      "node_modules/bar/index.js",
      `
const { prefix } = require("./prefix");
exports.foo = prefix + "foo";
`
    )
    .file(
      "node_modules/bar/index.ts",
      `
throw new Error("This should not be loaded");
`
    );
};

describe("import ESM from node_modules", async () => {
  it(".mts -> mod", async () => {
    const filePath = await addESMPackage(tmp())
      .file(
        "entry.mts",
        `
import { foo } from "foo";
console.log(foo);
`
      )
      .path("entry.mts");
    const result = await runNode(filePath);
    expect(result).toBe("This is foo\n");
  });
  it(".mts -> .mts -> mod", async () => {
    const filePath = await addESMPackage(tmp())
      .file(
        "loader.mts",
        `
import { foo } from "foo";
export const repeated: string = foo.repeat(3);
`
      )
      .file(
        "entry.mts",
        `
import { repeated } from "./loader.mjs";
console.log(repeated);
`
      )
      .path("entry.mts");
    const result = await runNode(filePath);
    expect(result).toBe("This is fooThis is fooThis is foo\n");
  });
  it(".mts -> .cts -> mod", async () => {
    const filePath = await addESMPackage(tmp())
      .file(
        "loader.cts",
        `
import { foo } from "foo";
export const repeated: Promise<string> = import("foo").then(({ foo }) => foo.repeat(3));
`
      )
      .file(
        "entry.mts",
        `
import mod from "./loader.cjs";
console.log(await mod.repeated);
`
      )
      .path("entry.mts");
    const result = await runNode(filePath);
    expect(result).toBe("This is fooThis is fooThis is foo\n");
  });
  it("does not confuse with local files", async () => {
    const filePath = await addESMPackage(tmp())
      .file(
        "foo.ts",
        `
throw new Error("This should not be loaded");
`
      )
      .file(
        "loader.cts",
        `
import { foo } from "foo";
export const repeated: Promise<string> = import("foo").then(({ foo }) => foo.repeat(3));
`
      )
      .file(
        "entry.mts",
        `
import mod from "./loader.cjs";
console.log(await mod.repeated);
`
      )
      .path("entry.mts");
    const result = await runNode(filePath);
    expect(result).toBe("This is fooThis is fooThis is foo\n");
  });
});

describe("import CJS from node_modules", async () => {
  it(".mts -> mod", async () => {
    const filePath = await addCJSPackage(tmp())
      .file(
        "entry.mts",
        `
import { foo } from "bar";
console.log(foo);
`
      )
      .path("entry.mts");
    const result = await runNode(filePath);
    expect(result).toBe("This is foo\n");
  });
  it(".mts -> .mts -> mod", async () => {
    const filePath = await addCJSPackage(tmp())
      .file(
        "loader.mts",
        `
import { foo } from "bar";
export const repeated: string = foo.repeat(3);
`
      )
      .file(
        "entry.mts",
        `
import { repeated } from "./loader.mjs";
console.log(repeated);
`
      )
      .path("entry.mts");
    const result = await runNode(filePath);
    expect(result).toBe("This is fooThis is fooThis is foo\n");
  });
  it(".mts -> .cts -> mod", async () => {
    const filePath = await addCJSPackage(tmp())
      .file(
        "loader.cts",
        `
import { foo } from "bar";
export const repeated: Promise<string> = import("bar").then(({ foo }) => foo.repeat(3));
`
      )
      .file(
        "entry.mts",
        `
import mod from "./loader.cjs";
console.log(await mod.repeated);
`
      )
      .path("entry.mts");
    const result = await runNode(filePath);
    expect(result).toBe("This is fooThis is fooThis is foo\n");
  });
  it("does not confuse with local files", async () => {
    const filePath = await addCJSPackage(tmp())
      .file(
        "foo.ts",
        `
throw new Error("This should not be loaded");
`
      )
      .file(
        "loader.cts",
        `
import { foo } from "bar";
export const repeated: Promise<string> = import("bar").then(({ foo }) => foo.repeat(3));
`
      )
      .file(
        "entry.mts",
        `
import mod from "./loader.cjs";
console.log(await mod.repeated);
`
      )
      .path("entry.mts");
    const result = await runNode(filePath);
    expect(result).toBe("This is fooThis is fooThis is foo\n");
  });
});
