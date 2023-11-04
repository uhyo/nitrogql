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
});
