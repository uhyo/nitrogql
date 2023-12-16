import { NitrogqlConfig, NitrogqlExtension } from "@nitrogql/cli";

const nitrogql: NitrogqlExtension = {
  generate: {
    mode: "with-loader-ts-5.0",
    schemaOutput: "./src/generated/schema.d.ts",
    name: {
      fragmentTypeSuffix: "Fragment",
      fragmentVariableSuffix: "Fragment",
    },
  },
};

const config: NitrogqlConfig = {
  schema: "./src/schema/pokeapi.json",
  documents: "./src/**/*.graphql",
  extensions: {
    nitrogql,
  },
};

export default config;
