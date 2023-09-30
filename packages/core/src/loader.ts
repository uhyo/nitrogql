/**
 * Loads given JS module as a schema and returns the schema as a SDL string.
 */
export async function loadSchemaJs(module: string): Promise<string> {
  let loaded = await import(module);
  while (loaded?.default) {
    loaded = loaded.default;
  }
  if (typeof loaded === "string") {
    return loaded;
  }
  const graphql = await import("graphql").catch((err) => {
    console.error(
      "Failed to load 'graphql' package. If you want to use a GraphQLSchema object as a schema, you need to install 'graphql' package."
    );
    throw err;
  });
  if (graphql.isSchema(loaded)) {
    return graphql.printSchema(loaded);
  }
  throw new Error(
    `Failed to load schema from '${module}'. The module must export a string or a GraphQLSchema object.`
  );
}
