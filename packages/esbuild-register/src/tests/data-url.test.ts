import { expect, it, describe } from "vitest";
import { tmp } from "./fs.js";
import { runNode } from "./runFile.js";
import { pathToFileURL } from "url";

describe("dataUrlResolutionBase", async () => {
  it("Can import using relative path", async () => {
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
await import("data:text/javascript," + encodeURIComponent(\`
import { foo } from "./foo.mts";
console.log(foo.repeat(5));
\`));
      `
      )
      .path("entry.mts");
    const result = await runNode(filePath, {
      dataUrlResolutionBase: filePath,
    });
    expect(result).toBe("foofoofoofoofoo\n");
  });
});
