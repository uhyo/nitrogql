import { expect, it } from "vitest";
import { tmp } from "./fs.js";
import { runNode } from "./runFile.js";

it("single file .ts should work", async (test) => {
  const filePath = await tmp()
    .file(
      "single.ts",
      `
const foo: string = "foo";
console.log(foo.repeat(5));
`
    )
    .path("single.ts");
  const result = await runNode(filePath);
  expect(result).toBe("foofoofoofoofoo\n");
});

it("single file .cts should work", async (test) => {
  const filePath = await tmp()
    .file(
      "single.cts",
      `
const foo: string = "foo";
console.log(foo.repeat(5));
`
    )
    .path("single.cts");
  const result = await runNode(filePath);
  expect(result).toBe("foofoofoofoofoo\n");
});

it("single file .mts should work", async (test) => {
  const filePath = await tmp()
    .file(
      "single.mts",
      `
const foo: string = "foo";
console.log(foo.repeat(5));
`
    )
    .path("single.mts");
  const result = await runNode(filePath);
  expect(result).toBe("foofoofoofoofoo\n");
});
