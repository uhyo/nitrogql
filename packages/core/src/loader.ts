import { pathToFileURL } from "node:url";
import type { GraphQLNamedType } from "graphql";

export type LoadSchemaResult = {
  /**
   * The schema as a SDL string.
   */
  schema: string;
  /**
   * All extensions for types in the schema.
   */
  typeExtensions: Record<string, Record<string, unknown>>;
};

/**
 * Loads given JS module as a schema and returns the schema as a SDL string.
 */
export async function loadSchemaJs(
  schemaPath: string
): Promise<LoadSchemaResult> {
  let loaded = await import(pathToFileURL(schemaPath).toString());
  while (loaded?.default) {
    loaded = loaded.default;
  }
  if (typeof loaded === "string") {
    return { schema: loaded, typeExtensions: {} };
  }
  const graphql = await import("graphql").catch((err) => {
    console.error(
      "Failed to load 'graphql' package. If you want to use a GraphQLSchema object as a schema, you need to install 'graphql' package."
    );
    throw err;
  });
  if (graphql.isSchema(loaded)) {
    const typeExtensions = Object.fromEntries(
      Object.entries(loaded.getTypeMap()).flatMap(([name, type]) => {
        if (Object.keys(type.extensions).length === 0) {
          return [];
        }
        const extensions = {
          "nitrogql:kind": typeToKind(graphql, type),
          ...type.extensions,
        };
        return [[name, extensions]];
      })
    );
    return { schema: graphql.printSchema(loaded), typeExtensions };
  }
  throw new Error(
    `Failed to load schema from '${schemaPath}'. The module must export a string or a GraphQLSchema object.`
  );
}

function typeToKind(graphql: typeof import("graphql"), ty: GraphQLNamedType) {
  if (graphql.isScalarType(ty)) {
    return "scalar";
  }
  if (graphql.isObjectType(ty)) {
    return "object";
  }
  if (graphql.isInterfaceType(ty)) {
    return "interface";
  }
  if (graphql.isUnionType(ty)) {
    return "union";
  }
  if (graphql.isEnumType(ty)) {
    return "enum";
  }
  if (graphql.isInputObjectType(ty)) {
    return "input";
  }
  return ty satisfies never;
}
