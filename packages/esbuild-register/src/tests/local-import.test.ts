import { expect, it, describe } from "vitest";
import { tmp } from "./fs.js";
import { runNode } from "./runFile.js";

describe("import files locally", async () => {
  it(".mts -> .mts", async () => {
    const filePath = await tmp()
      .file(
        "foo.mts",
        `
export const foo: string = "foo";
`
      )
      .file(
        "entry.mts",
        `
import { foo } from "./foo.mjs";
console.log(foo.repeat(5));
`
      )
      .path("entry.mts");
    const result = await runNode(filePath);
    expect(result).toBe("foofoofoofoofoo\n");
  });
  it(".mts -> .cts", async () => {
    const filePath = await tmp()
      .file(
        "foo.cts",
        `
export const foo: string = "foo";
`
      )
      .file(
        "entry.mts",
        `
import foo from "./foo.cjs";
console.log(foo.foo.repeat(5));
`
      )
      .path("entry.mts");
    const result = await runNode(filePath);
    expect(result).toBe("foofoofoofoofoo\n");
  });
  it(".mts -> .cts -> .cts", async () => {
    const filePath = await tmp()
      .file(
        "pika.cts",
        `
export const name1: string = "pika";
`
      )
      .file(
        "chu.cts",
        `
import { name1 } from "./pika.cjs";
export const name2: string = name1 + "chu";
`
      )
      .file(
        "entry.mts",
        `
import mod from "./chu.cjs";
console.log(mod.name2);
`
      )
      .path("entry.mts");
    const result = await runNode(filePath);
    expect(result).toBe("pikachu\n");
  });
  it(".mts -> .cts -> .cts (without extension)", async () => {
    const filePath = await tmp()
      .file(
        "pika.cts",
        `
export const name1: string = "pika";
`
      )
      .file(
        "chu.cts",
        `
import { name1 } from "./pika";
export const name2: string = name1 + "chu";
`
      )
      .file(
        "entry.mts",
        `
import mod from "./chu.cjs";
console.log(mod.name2);
`
      )
      .path("entry.mts");
    const result = await runNode(filePath);
    expect(result).toBe("pikachu\n");
  });
  it(".mts -> .cts -> .ts (without extension)", async () => {
    const filePath = await tmp()
      .file(
        "pika.ts",
        `
export const name1: string = "pika";
`
      )
      .file(
        "chu.cts",
        `
import { name1 } from "./pika";
export const name2: string = name1 + "chu";
`
      )
      .file(
        "entry.mts",
        `
import mod from "./chu.cjs";
console.log(mod.name2);
`
      )
      .path("entry.mts");
    const result = await runNode(filePath);
    expect(result).toBe("pikachu\n");
  });
});
