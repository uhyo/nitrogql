// Content mapper with a schema that mixes GraphQL files and a TypeScript schema file.

import { mkdir, rm, writeFile } from "node:fs/promises";
import { after, before, describe, it } from "node:test";
import assert from "node:assert/strict";

import { startMapper, tsPath } from "./mapper.mjs";

// Inside the package, so that the schema file can import `graphql`.
const projectUrl = new URL(".tmp/js-schema/", import.meta.url);
const projectDir = tsPath(projectUrl);
const tsconfig = tsPath(new URL("tsconfig.json", projectUrl));
const schemaFile = tsPath(new URL("schema.graphql", projectUrl));
const scalarsFile = new URL("scalars.ts", projectUrl);

/**
 * Returns a TypeScript schema file that defines the `Stamp` scalar.
 */
function scalars(codegenScalarType) {
  return `import { GraphQLScalarType, GraphQLSchema } from "graphql";

const Stamp = new GraphQLScalarType({
  name: "Stamp",
  extensions: { codegenScalarType: ${JSON.stringify(codegenScalarType)} },
});
export default new GraphQLSchema({ types: [Stamp] });
`;
}

describe(
  "content mapper with a TypeScript schema file",
  { timeout: 60_000 },
  () => {
    /** @type {ReturnType<typeof startMapper>} */
    let mapper;
    /** @type {string} */
    let configIdentity;
    before(async () => {
      await rm(projectUrl, { recursive: true, force: true });
      await mkdir(projectUrl, { recursive: true });
      await writeFile(new URL("tsconfig.json", projectUrl), "{}\n");
      await writeFile(
        new URL("graphql.config.yaml", projectUrl),
        `schema:
  - ./schema.graphql
  - ./scalars.ts
extensions:
  nitrogql:
    plugins:
      - "nitrogql:graphql-scalars-plugin"
`,
      );
      await writeFile(
        new URL("schema.graphql", projectUrl),
        "type Query {\n  now: Stamp!\n}\n",
      );
      await writeFile(scalarsFile, scalars("string"));
      mapper = startMapper(projectDir);
      await mapper.request("initialize", { positionEncodings: ["utf-16"] });
    });
    after(async () => {
      mapper.child.kill();
      await rm(projectUrl, { recursive: true, force: true });
    });

    /**
     * Opens the project and returns the schema types emitted from the GraphQL schema file.
     */
    async function openAndTransformSchema() {
      const open = await mapper.request("openProject", {
        configFileName: tsconfig,
        projectHandle: "project",
      });
      assert.equal(open.error, undefined);
      const transform = await mapper.request("transform", {
        fileName: schemaFile,
        content: "type Query {\n  now: Stamp!\n}\n",
        projectHandle: "project",
      });
      assert.equal(transform.result.diagnostics, undefined);
      return { open: open.result, text: transform.result.text };
    }

    it("provides schema types with types of scalars from the plugin", async () => {
      const { open, text } = await openAndTransformSchema();
      configIdentity = open.configIdentity;
      assert.match(text, /Stamp = string/);
    });

    it("loads the changed schema file when the project is reopened", async () => {
      await mapper.request("closeProject", { projectHandle: "project" });
      // Only type extensions change; the printed schema stays the same.
      await writeFile(scalarsFile, scalars("number"));
      const { open, text } = await openAndTransformSchema();
      assert.match(text, /Stamp = number/);
      // Results of operation files depend on the types of scalars.
      assert.notEqual(open.configIdentity, configIdentity);
    });
  },
);
