// @ts-check

/**
 * @type {import('@nitrogql/cli').NitrogqlConfig}
 */
const config = {
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
