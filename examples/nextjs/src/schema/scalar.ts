import { buildSchema, isSchema } from "graphql";
import { DateTimeTypeDefinition } from "graphql-scalars";

// Note: this is just for demonstrating `.ts` schema files.
// This may not be the best way to use graphql-scalars.
const schema = buildSchema(DateTimeTypeDefinition);
export default schema;
