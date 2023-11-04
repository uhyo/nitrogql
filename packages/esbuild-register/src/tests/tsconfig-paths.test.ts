import { expect, it, describe } from "vitest";
import { tmp } from "./fs.js";
import { runNode } from "./runFile.js";

const tsConfig = {
  compilerOptions: {
    paths: {
      "~/*": ["./util/*"],
    },
  },
};

describe("tsconfig.json 'paths' support", async () => {
  it(".mts -> .mts", async () => {
    const filePath = await tmp()
      .file("tsconfig.json", JSON.stringify(tsConfig))
      .dir("util")
      .file("util/foo.mts", `export const foo: string = "foo";`)
      .file(
        "entry.mts",
        `
import { foo } from "~/foo.mjs";
console.log(foo.repeat(5));
`
      )
      .path("entry.mts");
    const result = await runNode(filePath);
    expect(result).toBe("foofoofoofoofoo\n");
  });
  it(".mts -> .cts", async () => {
    const filePath = await tmp()
      .file("tsconfig.json", JSON.stringify(tsConfig))
      .dir("util")
      .file("util/foo.cts", `export const foo: string = "foo";`)
      .file(
        "entry.mts",
        `
import pkg from "~/foo.cjs";
const { foo } = pkg;
console.log(foo.repeat(5));
`
      )
      .path("entry.mts");
    const result = await runNode(filePath);
    expect(result).toBe("foofoofoofoofoo\n");
  });
  it(".mts -> .cts (without extension)", async () => {
    const filePath = await tmp()
      .file("tsconfig.json", JSON.stringify(tsConfig))
      .dir("util")
      .file("util/foo.cts", `export const foo: string = "foo";`)
      .file(
        "entry.mts",
        `
import pkg from "~/foo";
const { foo } = pkg;
console.log(foo.repeat(5));
`
      )
      .path("entry.mts");
    const result = await runNode(filePath);
    expect(result).toBe("foofoofoofoofoo\n");
  });
  it(".mts -> .ts -> .cts", async () => {
    const filePath = await tmp()
      .file("tsconfig.json", JSON.stringify(tsConfig))
      .dir("util")
      .file("util/foo.cts", `export const foo: string = "foo";`)
      .file(
        "loader.ts",
        `
import { foo } from "~/foo.cjs";
export { foo as foo2 };
`
      )
      .file(
        "entry.mts",
        `
import pkg from "./loader.js";
const { foo2 } = pkg;
console.log(foo2.repeat(5));
`
      )
      .path("entry.mts");
    const result = await runNode(filePath);
    expect(result).toBe("foofoofoofoofoo\n");
  });
});
