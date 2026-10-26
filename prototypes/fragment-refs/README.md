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

Fields omitted by `@skip`/`@include` are printed as `key?: never` in unrendered selections too.
Merging maps over the keys of both sides, which keeps a field optional only if both omit it,
and rendering is a single homomorphic mapped type that keeps the modifiers.

## Running

```sh
npm install
./run.sh                               # generate the cases and type-check them with TypeScript 7.1
TYPESCRIPT_API=typescript6 node dump-types.cjs <case>  # show the resolved types of a case
node measure.cjs [case]                # compare costs of resolving the types (TypeScript 6 API)
./bench.sh [operations]                # compare type checking costs with TypeScript 7
```

Cases are defined in `crates/printer/src/operation_type_printer/tests/fragment_refs.rs`.
Each case is printed with all fragments expanded into one module (`expanded.ts`), and with
fragments in separate modules (`main.ts` and others). `check.ts` asserts that the types are
identical, using the strict `Equal` check.

## Findings

All 31 cases pass with TypeScript 7.1.0-beta.

- **Identical types** (29 cases): fragments on interfaces spread on object types and the other
  way around, nested and list merging, nullable lists, inline fragments inside fragments,
  `__typename`, aliases, chains of fragments across modules, local fragments that spread
  imported ones, `@skip`/`@include` on spreads (constant and variable), variables used inside
  fragments, and the stress and colocation benchmarks.
- **Wider types** (2 cases):
  - When the operation and a fragment use the same variable for `@skip`/`@include`, the
    reference version cannot tell that the values are the same. It includes combinations
    that cannot happen. The result is still a supertype of the real one, so it is sound.
  - `variables_in_nested_merging` shows a bug in the **current** printer:
    `merge_selection_trees` pairs each branch with only the first branch of the same object
    type. Combinations are missing, e.g. `age` is typed as absent even when `$y` is true.
    The reference version gets this right.
- **Cost**: resolving the types takes more type instantiations. See below.

## Type checking cost

`bench.sh` generates a colocation-style project: 200 operations, each spreading two
`UserCard` fragments, each of which spreads two `PostItem` fragments (600 fragment modules).
Every result type is fully resolved, so this is the worst case: in real code, TypeScript only
resolves what is used.

| Helpers | Instantiations | Check time (TypeScript 7) |
| --- | --- | --- |
| Expanded (current printer) | 145k | 0.16–0.19s |
| First version | 788k | 1.1–1.9s |
| Current version | 477k | 0.74s |

Measured in the same run. Timings on this machine vary between runs by up to 2×; another run
measured 0.33s expanded against 1.1s referred. Instantiations are stable.

What made the difference:

- Omitted fields as `key?: never` instead of a marker type, so that rendering does not need
  key remapping (`as` clauses) or a check per field, and not checking for `any` while
  rendering, as `any` absorbs the union anyway: −32%.
- Merging fields with a plain intersection when the two sides share no keys, and finding the
  matching branches once instead of twice: −10%.

Tried without gain: an intersection fast path for overlaps on leaves only (checking costs more
than it saves).

Merging fragments one at a time nests instantiations, and 50 spreads in one selection
failed with TS2589. Combining them as a balanced tree removes that limit (tested up to 120).

## Not covered yet

- Validation: the checker still needs imported fragments for spread applicability, variables
  used in fragments, and overlapping fields.
- Wiring into the content mapper: emitting the imports from `#import` (`.graphql` specifiers;
  wildcard imports need namespace imports), removing imported contents from `config_identity`.
- Fallback: a fragment with errors exports `any` as its tree, which makes branches it is merged
  into `any`. No case covers this yet.
- Spreads of fragments that come from transitive imports (the current resolver makes them
  visible to the importing file).
