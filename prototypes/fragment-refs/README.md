# Prototype: referring to fragments in other modules

The content mapper currently expands `#import`ed fragments into each importing file. As a
result, the output of a file depends on the contents of other files, and the mapper has to
track those dependencies itself (`config_identity`, watching every operation file).

This prototype checks whether types can instead *refer* to fragments in other modules through
TypeScript imports, so that TypeScript tracks the dependencies, while keeping types the same.

## How it works

With `OperationTypePrinterOptions::external_fragments`, spreads of fragments defined in other
modules are not expanded. The printer imports their selection trees and prints selections in
an unrendered form instead:

```ts
import type { __NitrogqlTree_F } from "./f";

export type TestResult = Schema.__Render<Schema.__MergeFragment<
  Schema.__Branch<"Query", Schema.__OperationOutput.Query, { me: Schema.__Branch<"User", ...> }, {}>,
  __NitrogqlTree_F
>>;
```

With `export_fragment_trees`, each fragment also exports its selection tree
(`__NitrogqlTree_<Name>`), a union of `__Branch` types for each concrete object type and
combination of variable values.

Utility types are emitted into the schema module with
`SchemaTypePrinterOptions::emit_fragment_merge_utilities`:

- `__MergeFragment` merges a fragment's fields into branches of the same object type, deeply.
- `__CombineFragments` combines the fragments spread in one selection. The printer combines them
  as a balanced tree, because TypeScript limits the depth of nested instantiations (TS2589).
- `__SkipFragment` handles `@skip`/`@include` on spreads.
- `__Render` turns the result into the same `__SelectionSet` types the expanding printer prints.

## Running

```sh
npm install
./run.sh                               # generate the cases and type-check them with TypeScript 7.1
TYPESCRIPT_API=typescript6 node dump-types.cjs <case>  # show the resolved types of a case
node measure.cjs [case]                # compare costs of resolving the types (TypeScript 6 API)
```

Cases are defined in `crates/printer/src/operation_type_printer/tests/fragment_refs.rs`.
Each case is printed with all fragments expanded into one module (`expanded.ts`), and with
fragments in separate modules (`main.ts` and others). `check.ts` asserts that the types are
identical, using the strict `Equal` check.

## Findings

All 29 cases pass with TypeScript 7.1.0-beta.

- **Identical types** (27 cases): fragments on interfaces spread on object types and the other
  way around, nested and list merging, nullable lists, inline fragments inside fragments,
  `__typename`, aliases, chains of fragments across modules, local fragments that spread
  imported ones, `@skip`/`@include` on spreads (constant and variable), variables used inside
  fragments, and a stress case.
- **Wider types** (2 cases):
  - When the operation and a fragment use the same variable for `@skip`/`@include`, the
    reference version cannot tell that the values are the same. It includes combinations
    that cannot happen. The result is still a supertype of the real one, so it is sound.
  - `variables_in_nested_merging` shows a bug in the **current** printer:
    `merge_selection_trees` pairs each branch with only the first branch of the same object
    type. Combinations are missing, e.g. `age` is typed as absent even when `$y` is true.
    The reference version gets this right.
- **Cost**: resolving the types takes more type instantiations. In the stress case, where every
  fragment is merged into the same nested selections, each spread costs about 3,000–4,300
  instantiations, versus about 130 when expanded. Growth is linear: 120 spreads in one
  selection take about 0.5M instantiations, below TypeScript's limits.
  Without the balanced combining, 50 spreads in one selection already failed with TS2589.

## Not covered yet

- Validation: the checker still needs imported fragments for spread applicability, variables
  used in fragments, and overlapping fields.
- Wiring into the content mapper: emitting the imports from `#import` (`.graphql` specifiers;
  wildcard imports need namespace imports), removing imported contents from `config_identity`.
- Fallback: a fragment with errors exports `any` as its tree, which makes branches it is merged
  into `any`. No case covers this yet.
- Spreads of fragments that come from transitive imports (the current resolver makes them
  visible to the importing file).
