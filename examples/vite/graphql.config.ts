import { NitrogqlConfig } from "@nitrogql/cli";

const config: NitrogqlConfig = {
  schema: "./src/schema/pokeapi.json",
  documents: "./src/**/*.graphql",
  extensions: {
    nitrogql: {
      generate: {
        mode: "with-loader-ts-5.0",
        schemaOutput: "./src/generated/schema.d.ts",
      },
    },
  },
};

export default config;
