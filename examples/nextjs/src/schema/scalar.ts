import { GraphQLScalarType, GraphQLSchema } from "graphql";
import { DateResolver, DateTimeResolver } from "graphql-scalars";

/**
 * graphql-scalars types these scalars as `Date | string`.
 * This app passes ISO strings as they are, so they are typed as `string`.
 * The graphql-scalars plugin reads the type from `codegenScalarType`.
 */
function asString(scalar: GraphQLScalarType): GraphQLScalarType {
  return new GraphQLScalarType({
    ...scalar.toConfig(),
    extensions: { ...scalar.extensions, codegenScalarType: "string" },
  });
}

const schema = new GraphQLSchema({
  types: [asString(DateTimeResolver), asString(DateResolver)],
});
export default schema;
