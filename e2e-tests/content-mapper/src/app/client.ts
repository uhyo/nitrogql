import type { ResultOf, VariablesOf } from "@graphql-typed-document-node/core";
import getUserQuery from "./getUser.graphql";
import updateNameMutation from "./updateName.graphql";
import { UserFields } from "./userFields.graphql";

type Equals<A, B> =
  (<T>() => T extends A ? 1 : 2) extends <T>() => T extends B ? 1 : 2
    ? true
    : false;
function assertType<_ extends true>() {}

assertType<
  Equals<
    ResultOf<typeof getUserQuery>,
    {
      user: {
        id: string;
        name: string;
        age: number | null;
        role: "ADMIN" | "USER";
      } | null;
    }
  >
>();
assertType<
  Equals<VariablesOf<typeof getUserQuery>, { readonly id: string | number }>
>();

assertType<
  Equals<
    VariablesOf<typeof updateNameMutation>,
    { readonly id: string | number; readonly name: string }
  >
>();

assertType<Equals<ResultOf<typeof UserFields>, { id: string; name: string }>>();

// @ts-expect-error -- age is not selected in UserFields
type _Age = ResultOf<typeof UserFields>["age"];
