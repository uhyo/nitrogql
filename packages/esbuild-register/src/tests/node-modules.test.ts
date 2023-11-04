import { expect, it, describe } from "vitest";
import { Folder, tmp } from "./fs.js";
import { runNode } from "./runFile.js";

const addPackage = (folder: Folder): Folder => {
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

describe("import from node_modules", async () => {
  it(".mts -> mod", async () => {
    const filePath = await addPackage(tmp())
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
    const filePath = await addPackage(tmp())
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
    const filePath = await addPackage(tmp())
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
