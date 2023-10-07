import { GraphQLSchema } from "graphql";
import { DateTimeResolver } from "graphql-scalars";

const schema = new GraphQLSchema({
  types: [DateTimeResolver],
});
export default schema;
