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

type GetMeResult = __Nitrogql.Render<
  __Nitrogql.Branch<
    "Query",
    Schema.__OperationOutput.Query,
    {
      me: __Nitrogql.MergeFragment<
        __Nitrogql.Branch<
          "User",
          Schema.__OperationOutput.User,
          { id: Schema.__OperationOutput.ID },
          {}
        >,
        __Nitrogql.Fragment<__NitrogqlImport0.__NitrogqlFragments, "UserFields">
      >;
    },
    {}
  >
>;
```

With `export_fragment_trees`, each module exports `__NitrogqlFragments`, which maps names of
its fragments to their selection trees: unions of `Branch` types for each concrete object
type and combination of variable values. Branches selected under values of variables carry
them as a fifth type argument, such as `{ flag: true }`. Wildcard imports look fragments up in all wildcard
imported modules; `Fragment` gives `any` for fragments that a module does not provide.

The utility types are in `packages/cli/fragment-types.d.ts`, shipped with `@nitrogql/cli`
so that they always match the printer:

- `MergeFragment` merges a fragment's fields into branches of the same object type, deeply.
  Branches whose values of a shared variable differ are not merged, as they cannot happen
  together.
- `CombineFragments` combines the fragments spread in one selection. The printer combines them
  as a balanced tree, because TypeScript limits the depth of nested instantiations (TS2589).
- `SkipFragment` handles `@skip`/`@include` on spreads.
- `Render` turns the result into the same types the expanding printer prints.

Fields omitted by `@skip`/`@include` are printed as `key?: never` in unrendered selections too.
Merging maps over the keys of both sides, which keeps a field optional only if both omit it,
and rendering is a single homomorphic mapped type that keeps the modifiers.

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

## Findings

All 37 cases pass with TypeScript 7.1.0-beta.

- **Identical types** (all cases): fragments on interfaces spread on object types and the other
  way around, nested and list merging, nullable lists, inline fragments inside fragments,
  `__typename`, aliases, chains of fragments across modules, local fragments that spread
  imported ones, `@skip`/`@include` on spreads (constant and variable), variables used inside
  fragments, the same variable used in the operation and in fragments (directly, nested, in
  chains of fragments and on skipped spreads), and the stress and colocation benchmarks.
- **Cost**: resolving the types takes more type instantiations. See below.

## Type checking cost

`bench.sh` generates a colocation-style project: 200 operations, each spreading two
`UserCard` fragments, each of which spreads two `PostItem` fragments (600 fragment modules).
Every result type is fully resolved, so this is the worst case: in real code, TypeScript only
resolves what is used.

| Helpers                    | Instantiations | Check time (TypeScript 7) |
| -------------------------- | -------------- | ------------------------- |
| Expanded (current printer) | 145k           | 0.16–0.19s                |
| First version              | 788k           | 1.1–1.9s                  |
| Current version            | 477k           | 0.74s                     |

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

### Variables

`bench.sh` also runs `colocation_variables`, the same project where the operations and both
kinds of fragments include fields by the same `$flag`. Merging only branches that agree on
shared variables (#128) measured as follows, in the same run:

| Case                   | Expanded | Ignoring variables (wider) | Matching variables |
| ---------------------- | -------- | -------------------------- | ------------------ |
| `colocation`           | 179k     | 539k                       | 542k               |
| `colocation_variables` | 374k     | 2,356k                     | 1,051k             |

Without variables, the previous helpers measured 528k, so carrying variables costs +2.7%
(about half of it from intersecting the variables of merged branches). With shared variables, matching them halves the cost, as
combinations that cannot happen are no longer built.

The pattern for agreeing branches is `{ " $vars": Partial<Vars> & { [name: string]: boolean } }`.
Without the index signature, the pattern is a weak type, which rejects branches that share no
variables with it. Tried without gain: excluding branches that match a union of conflicting
values (552k and 1,113k), and skipping the intersection when one side has no variables.

Merging fragments one at a time nests instantiations, and 50 spreads in one selection
failed with TS2589. Combining them as a balanced tree removes that limit (tested up to 120).

## Not covered

- A fragment with errors exports `any` as its tree, which makes branches it is merged into
  `any`. This is not compared with expanded types, which make the whole definition `any`.
- Fragments that a file spreads without importing them, relying on an imported file importing
  them (`nitrogql check` resolves imports transitively): the content mapper reports them as
  unknown fragments, or types them as `any` when the file has wildcard imports.
