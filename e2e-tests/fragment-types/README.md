# Fragment imports: type equality harness

The content mapper does not expand `#import`ed fragments into importing files. Instead, each
module exports selection trees of its fragments, and importing modules merge them at the type
level. This way, the output of a file does not depend on contents of other files, and
TypeScript tracks the dependencies.

This directory checks that the merged types are identical to the types printed for fully
expanded selections, and measures the cost of type checking them.

## How it works

With `OperationTypePrinterOptions::fragment_imports`, spreads of imported fragments are not
expanded. The printer imports the modules as namespaces and prints selections in an
unrendered form:

```ts
import type * as __Nitrogql from "@nitrogql/cli/fragment-types";
import type * as __NitrogqlImport0 from "./userFields.graphql";

type GetMeResult = __Nitrogql.Render<{
  Query: __Nitrogql.Branch<
    {},
    {
      me: {
        User: __Nitrogql.MergeLeaves<
          __Nitrogql.At<
            __Nitrogql.Fragment<
              __NitrogqlImport0.__NitrogqlFragments,
              "UserFields"
            >,
            "User"
          >,
          { id: Schema.__OperationOutput.ID },
          never
        >;
      };
    },
    never
  >;
}>;
```

An unrendered selection is a record that maps names of concrete object types to `Branch` types,
or to unions of them when branches of a type are selected under different values of variables.
`Branch<Leaves, Objects, Omitted, Vars>` has the fields of one branch, classified by
the printer:

- `Leaves`: fields of scalar and enum types (aliased or not). Fields with the same response key
  have the same type, so leaves are merged by intersecting them.
- `Objects`: fields of object types, whose values are unrendered selections, possibly in lists
  or nullable. These are the only fields that are merged deeply.
- `Omitted`: the union of names of fields omitted by `@skip`/`@include`.
- `Vars`: the union of values of variables the branch is selected under, such as
  `"flag=true" | "other=false"`.

With `export_fragment_trees`, each module exports `__NitrogqlFragments`, which maps names of
its fragments to their selections. Wildcard imports look fragments up in all wildcard imported
modules; `Fragment` gives `any` for fragments that a module does not provide.

The utility types are in `packages/cli/fragment-types.d.ts`, shipped with `@nitrogql/cli`
so that they always match the printer. The printer merges fragments into each branch of a
selection with the simplest one that applies, as it knows the branch:

- `At` gives the branches of one object type in a fragment's selection.
- `Spread` merges fragments into a branch without fields of its own: the result is the
  fragments' branches.
- `MergeLeaves` merges fragments into a branch that has only leaf fields.
- `MergeFragment` merges fragments into any other branch. Branches whose values of a shared
  variable differ are not merged, as they cannot happen together.
- `CombineFragments` combines the branches of fragments spread in one selection. The printer
  combines them as a balanced tree, because TypeScript limits the depth of nested
  instantiations (TS2589).
- `SkipFragment` handles `@skip`/`@include` on spreads.
- `Render` turns the result into types identical to the ones the expanding printer prints.

Expanded types are object types with the fields of each branch: unaliased fields in the order
of the selection, then aliased ones. `Render` flattens the fields of each branch, which gives
identical types (the order of fields does not matter to type relations), but hovers show leaves
before object fields. Omitted fields are optional `never` in both, as they are not in the data.

## Running

CI runs the check as part of the e2e tests. To run it locally (Rust and Node are needed):

```sh
npm install                            # in the repository root
npm run e2e-test                       # generate the cases and type-check them with TypeScript 7.1
TYPESCRIPT_API=typescript6 node dump-types.cjs <case>  # show the resolved types of a case
node measure.cjs [case]                # compare costs of resolving the types (TypeScript 6 API)
./bench.sh [operations]                # compare type checking costs with TypeScript 7
```

Cases are defined in `crates/printer/src/operation_type_printer/tests/fragment_refs.rs`.
Each case is printed with all fragments expanded into one module (`expanded.ts`), and with
fragments in separate modules (`main.ts` and others), imported with specific and wildcard
imports. `check.ts` asserts that the types are identical, using the strict `Equal` check.
In the small cases, each module imports all other modules, so that lookups through several
wildcard imports are covered. The benchmark cases import only the modules they spread
fragments of, as real modules do.

## Findings

All 39 cases pass with TypeScript 7.1.0-beta, also with `exactOptionalPropertyTypes`.

- **Identical types** (all cases): fragments on interfaces spread on object types and the other
  way around, nested and list merging, nullable lists, inline fragments inside fragments,
  `__typename`, aliases (also to the field's own name), chains of fragments across modules, local fragments that spread
  imported ones, `@skip`/`@include` on spreads (constant and variable), variables used inside
  fragments, the same variable used in the operation and in fragments (directly, nested, in
  chains of fragments and on skipped spreads), and the benchmark cases.
- **Cost**: resolving the types takes more type instantiations. See below.

## Type checking cost

`bench.sh` generates three projects and fully resolves every result type, so this is the worst
case: in real code, TypeScript only resolves what is used.

- `colocation`: 200 operations, each spreading two `UserCard` fragments, each of which spreads
  two `PostItem` fragments (600 fragment modules).
- `colocation_variables`: the same, where the operations and both kinds of fragments include
  fields by the same `$flag`.
- `wide`: a schema shaped like larger ones, where object types have 30 fields and an interface
  has 20 implementations. 200 operations select from both and spread a fragment on the object
  type and two fragments on the interface.

Instantiations and check time with TypeScript 7.1, measured in the same session:

| Case                   | Expanded\*   | Before (ea4f21c) | Current        |
| ---------------------- | ------------ | ---------------- | -------------- |
| `colocation`           | 145k / 0.12s | 579k / 0.36s     | 287k / 0.20s   |
| `colocation_variables` | 340k / 0.25s | 1,148k / 0.68s   | 636k / 0.41s   |
| `wide`                 | 536k / 0.44s | 3,694k / 1.9s    | 1,271k / 0.64s |

\* Expanded types as they were printed then, with `__SelectionSet` (see below).

Timings vary between runs by about 10%; instantiations are stable. Checking only the modules,
without resolving the result types, cost 0.44s before and 0.34s now for 1,000 colocation
operations (0.19s expanded). In `examples/nextjs`, `scripts/bench-fragments.mjs` measured
253k instantiations before and 153k now, against 257k with fragments inlined.

What made the difference, measured with 1,000 operations:

- Rendering by flattening fields instead of with `__SelectionSet`, which visits every field of
  the schema type and costs more for larger types: about −20% time. The schema type is no longer
  needed in branches, which saved another 10% of instantiations.
- Selections as records keyed by object types, instead of unions of branches tagged with their
  type names. Merging looked matching branches up with `Extract`, which compares each branch of
  one side with each of the other, so selections of abstract types cost quadratically in the
  number of their implementations: in a synthetic test, going from 5 to 50 implementations
  multiplied the cost by 23, against 10 now (and 11 for expanded types). The printer also looks
  up the branches of each object type in fragments (`At`), so that local selections are not
  merged as a whole: −15% for `colocation`.
- Values of variables as unions of string literals (`"flag=true"`) instead of objects. Branches
  that conflict are found by intersecting the values with negated ones (computed once for each
  combination), instead of relating them to `Partial<Vars> & { [name: string]: boolean }`:
  −19% for `colocation_variables`.
- Classifying fields when printing and merging into branches with only leaves with `Spread` and
  `MergeLeaves`, which skip merging the parts that the branch does not have: −5% to −20% (most
  when only checking the modules).

Tried without gain: checking `any` with `unknown extends A | B` in one condition (building the
union costs more than the condition it saves), classifying aliased fields separately, and
resolving list elements with `T[number]` instead of `infer`.

Before #128, merging ignored variables, which built combinations that cannot happen and cost
twice as much for `colocation_variables`.

### Expanded types

Expanded types were printed with `__SelectionSet<Orig, Fields, Aliased>` of schema types, which
picks the fields from the schema type `Orig` so that hovers show the descriptions of the fields
in the schema. This visits every field of the schema type, like rendering did before. The
printer now prints plain object types and copies the descriptions (and deprecation reasons) of
unaliased fields to them.

`__SelectionSet` also typed omitted unaliased fields as required `undefined` (`never` with
`exactOptionalPropertyTypes`), as it kept the modifiers of the schema type. They are now
optional `never` like omitted aliased fields, so `Branch` no longer needs the names of aliased
fields, which it had only for typing omitted fields differently.

Measured in the same session, with TypeScript 7.1:

| Case                   | Expanded, before | Expanded, now | Referred, before | Referred, now  |
| ---------------------- | ---------------- | ------------- | ---------------- | -------------- |
| `colocation`           | 145k / 0.11s     | 28k / 0.04s   | 288k / 0.16s     | 269k / 0.20s   |
| `colocation_variables` | 340k / 0.23s     | 67k / 0.07s   | 639k / 0.38s     | 600k / 0.34s   |
| `wide`                 | 536k / 0.41s     | 86k / 0.07s   | 1,279k / 0.61s   | 1,123k / 0.50s |

Checking only the expanded modules for 1,000 colocation operations, without resolving the
result types, takes 51 instantiations and 0.07s (0.34s referred). In `examples/nextjs`,
`scripts/bench-fragments.mjs` measured 249k instantiations (0.31s) with fragments inlined before,
and 92k (0.19s) now, against 137k (0.22–0.30s between runs) with imported fragments, which took
150k (0.24s) before.

As each branch of a union has its own declaration of a field, hovers on a deprecated field of a
union list the `@deprecated` tag once for each branch.

Merging fragments one at a time nests instantiations, and 50 spreads in one selection
failed with TS2589. Combining them as a balanced tree removes that limit (tested up to 120).

## Not covered

- A fragment with errors exports `any` as its selection, which makes branches it is merged into
  `any`. This is not compared with expanded types, which make the whole definition `any`;
  `any.ts` checks that each utility gives `any` for such branches instead.
- Fragments that a file spreads without importing them, relying on an imported file importing
  them (`nitrogql check` resolves imports transitively): the content mapper reports them as
  unknown fragments, or types them as `any` when the file has wildcard imports.
