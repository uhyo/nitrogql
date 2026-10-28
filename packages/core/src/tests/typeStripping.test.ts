import { mkdir, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { afterEach, beforeAll, describe, expect, it, vi } from "vitest";

// Workers run the built server, so the built code is tested.
const dist = new URL("../../dist/", import.meta.url);
const { getCommandClient }: typeof import("../command/commandClient.js") =
  await import(new URL("command/commandClient.js", dist).href);
const { executeNodeSync }: typeof import("../config.js") = await import(
  new URL("config.js", dist).href
);

const nodeHasTypeStripping = !!(
  process.features as { typescript?: string | false }
).typescript;

let dir: string;

beforeAll(async () => {
  dir = path.join(
    tmpdir(),
    "nitrogql-core-" + Date.now() + "-" + Math.random().toString(36).slice(2),
  );
  await mkdir(dir, { recursive: true });
  await writeFile(
    path.join(dir, "package.json"),
    JSON.stringify({ type: "module" }),
  );
  await writeFile(
    path.join(dir, "lib.ts"),
    `const foo: string = "foo";\nexport default foo;\n`,
  );
  await writeFile(
    path.join(dir, "entry.ts"),
    `import foo from "./lib.ts";\nexport default foo;\n`,
  );
  // Importing .ts files by .js works only with @nitrogql/esbuild-register.
  await writeFile(
    path.join(dir, "mapped.ts"),
    `import foo from "./lib.js";\nexport default foo;\n`,
  );
});

afterEach(() => {
  vi.unstubAllEnvs();
});

function command(file: string): string {
  return `
import graphql from "graphql";
const { default: value } = await import(${JSON.stringify(
    pathToFileURL(path.join(dir, file)).toString(),
  )});
export default {
  value,
  graphql: typeof graphql.parse,
  esbuildRegister: process.execArgv.some((arg) => arg.includes("esbuild-register")),
};
`;
}

describe("command client", () => {
  it.skipIf(!nodeHasTypeStripping)(
    "uses Node.js type stripping by default",
    async () => {
      const client = getCommandClient();
      try {
        expect(await client.run(command("entry.ts"))).toEqual({
          value: "foo",
          graphql: "function",
          esbuildRegister: false,
        });
      } finally {
        client.close();
      }
    },
  );

  it("uses @nitrogql/esbuild-register when opted out", async () => {
    vi.stubEnv("NITROGQL_DISABLE_TYPE_STRIPPING", "1");
    const client = getCommandClient();
    try {
      expect(await client.run(command("mapped.ts"))).toEqual({
        value: "foo",
        graphql: "function",
        esbuildRegister: true,
      });
    } finally {
      client.close();
    }
  });
});

describe("executeNodeSync", () => {
  const code = (file: string) => `
const { default: foo } = await import(${JSON.stringify(
    pathToFileURL(path.join(dir, file)).toString(),
  )});
process.stdout.write(foo);
`;

  it.skipIf(!nodeHasTypeStripping)(
    "uses Node.js type stripping by default",
    () => {
      expect(executeNodeSync(code("entry.ts"))).toBe("foo");
      // .js is not mapped to .ts without @nitrogql/esbuild-register.
      expect(() => executeNodeSync(code("mapped.ts"))).toThrow();
    },
  );

  it("uses @nitrogql/esbuild-register when opted out", () => {
    vi.stubEnv("NITROGQL_DISABLE_TYPE_STRIPPING", "1");
    expect(executeNodeSync(code("mapped.ts"))).toBe("foo");
  });
});
