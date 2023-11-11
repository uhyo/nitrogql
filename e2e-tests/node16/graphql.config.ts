import { NitrogqlConfig } from "@nitrogql/cli";

const config: NitrogqlConfig = {
  schema: ["./src/schema/*.ts", "./src/schema/*.graphql"],
  documents: "./src/app/*.graphql",
  extensions: {
    nitrogql: {
      plugins: ["nitrogql:model-plugin", "nitrogql:graphql-scalars-plugin"],
      generate: {
        mode: "with-loader-ts-5.0",
        schemaOutput: "./src/generated/schema.d.ts",
        resolversOutput: "./src/generated/resolvers.d.ts",
      },
    },
  },
};

export default config;
