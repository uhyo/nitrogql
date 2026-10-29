// Utility types for merging fragments imported with `#import`, used by the content mapper.
//
// Selections that spread imported fragments are printed unrendered: a selection is a record
// that maps names of concrete object types to `Branch` types with their fields. Fields of
// fragments are merged into branches of the same object type, and `Render` converts the result
// into the same types as the ones printed for fully expanded selections.
//
// The printer does as much as it can when printing: it classifies fields by kind, and merges
// fragments into each branch of a selection with the simplest utility that applies, so that
// TypeScript does not need to inspect what the printer already knows.

/**
 * Selection of one concrete object type.
 * Fields are classified when printed, so that merging does not inspect them:
 *
 * - `Leaves` has fields of scalar and enum types. Fields of the same name have the same type,
 *   so they are merged by intersecting.
 * - `Objects` has fields of object types, whose values are unrendered selections, possibly
 *   in lists or nullable.
 * - `Omitted` is the union of names of fields omitted by `@skip` or `@include`.
 *   A field omitted on one side and selected on the other is selected.
 * - `Aliased` is the union of names of aliased fields, which are typed differently when omitted.
 * - `Vars` is the union of values of boolean variables (used in `@skip` or `@include`) that
 *   the branch is selected with, such as `"flag=true" | "other=false"`.
 */
export type Branch<Leaves, Objects, Omitted, Aliased, Vars = never> = {
  " $leaves": Leaves;
  " $objects": Objects;
  " $omitted": Omitted;
  " $aliased": Aliased;
  " $vars": Vars;
};
type AnyBranch = Branch<unknown, unknown, unknown, unknown, unknown>;
type EmptyBranch = Branch<{}, {}, never, never>;

/**
 * Selection tree of an imported fragment, or `any` if the module does not provide it
 * (for example because it has errors).
 */
export type Fragment<
  Fragments,
  Name extends string,
> = Name extends keyof Fragments ? Fragments[Name] : any;

/** Branches of given object type in a selection, or `never` if the selection has none. */
export type At<Selection, Name extends string> = Name extends keyof Selection
  ? Selection[Name]
  : never;

// Selections of the same field have branches of the same object types.
// The left side decides which branches the result has.
type MergeSelections<A, B> = {
  [K in keyof A]: K extends keyof B ? MergeBranches<A[K], B[K]> : A[K];
};
// Values of object fields: selections, possibly in lists or nullable.
type MergeValue<A, B> = A extends readonly unknown[]
  ? MergeValue<A[number], B extends readonly unknown[] ? B[number] : never>[]
  : A extends null
    ? A
    : MergeSelections<A, B & {}>;
// Only fields that both sides have are merged deeply.
type MergeObjects<A, B> = [keyof A & keyof B] extends [never]
  ? A & B
  : {
      [K in keyof A | keyof B]: K extends keyof A
        ? K extends keyof B
          ? MergeValue<A[K], B[K]>
          : A[K]
        : B[K & keyof B];
    };

// Distributes over branches of one object type, which differ in variable values.
// Branches of the other side that do not agree on the values of variables both have are
// not merged, as they cannot happen together. Variables that one side does not have may have
// any value on the other.
// Branches merged from fragments with errors are `any`, and so are the results of merging them.
type MergeBranches<A, B> = unknown extends A
  ? any
  : unknown extends B
    ? any
    : A extends AnyBranch
      ? MergeAgreeing<
          A,
          [A[" $vars"]] extends [never] ? B : Agreeing<B, Negate<A[" $vars"]>>
        >
      : never;
// Values that conflict with given values. Unions of the same values are the same type, so
// this is computed once for each combination of values.
type Negate<Vars> = Vars extends `${infer Name}=true`
  ? `${Name}=false`
  : Vars extends `${infer Name}=false`
    ? `${Name}=true`
    : never;
type Agreeing<B, Conflicts> = B extends AnyBranch
  ? [B[" $vars"] & Conflicts] extends [never]
    ? B
    : never
  : never;
type MergeAgreeing<A extends AnyBranch, B> = [B] extends [never]
  ? A
  : B extends AnyBranch
    ? Branch<
        A[" $leaves"] & B[" $leaves"],
        MergeObjects<A[" $objects"], B[" $objects"]>,
        A[" $omitted"] | B[" $omitted"],
        A[" $aliased"] | B[" $aliased"],
        A[" $vars"] | B[" $vars"]
      >
    : never;

/**
 * Combines branches of one object type from fragments spread in the same selection.
 * The printer combines them as a balanced tree, as TypeScript limits the depth of nested
 * instantiations.
 */
export type CombineFragments<A, B> = unknown extends A
  ? any
  : unknown extends B
    ? any
    : [A] extends [never]
      ? B
      : MergeBranches<A, B>;
/** Branches of a fragment spread that is skipped by `@skip` or `@include`. */
export type SkipFragment<B> = unknown extends B
  ? any
  : B extends AnyBranch
    ? Branch<
        {},
        {},
        B[" $omitted"] | keyof B[" $leaves"] | keyof B[" $objects"],
        B[" $aliased"],
        B[" $vars"]
      >
    : never;

/** Merges branches of fragments into a branch of the selection. */
export type MergeFragment<A, Fragment> = MergeBranches<A, Fragment>;
/**
 * Merges branches of fragments into a branch of the selection that has no fields of its own
 * and is not selected under values of variables: the result is the fragments' branches.
 */
export type Spread<Fragment> = unknown extends Fragment
  ? any
  : [Fragment] extends [never]
    ? EmptyBranch
    : Fragment;
/**
 * Merges branches of fragments into a branch of the selection that has only leaf fields
 * (selected or omitted) and is not selected under values of variables.
 */
export type MergeLeaves<Fragment, Leaves, Omitted, Aliased> =
  unknown extends Fragment
    ? any
    : [Fragment] extends [never]
      ? Branch<Leaves, {}, Omitted, Aliased>
      : AddLeaves<Fragment, Leaves, Omitted, Aliased>;
type AddLeaves<B, Leaves, Omitted, Aliased> = B extends AnyBranch
  ? Branch<
      Leaves & B[" $leaves"],
      B[" $objects"],
      Omitted | B[" $omitted"],
      Aliased | B[" $aliased"],
      B[" $vars"]
    >
  : never;

// Omitted fields are typed as in expanded selections: unaliased ones are `undefined` (or `never`
// with `exactOptionalPropertyTypes`, same as `__Omitted` of schema types), and aliased ones are
// optional `never`.
type OmittedValue = { key?: never } extends { key?: infer V } ? V : never;
// Fields omitted on one side of a merge and selected on the other are selected.
type OmittedFields<B extends AnyBranch> = [B[" $omitted"]] extends [never]
  ? {}
  : OmittedOnly<
      Exclude<B[" $omitted"], keyof B[" $leaves"] | keyof B[" $objects"]>,
      B[" $aliased"]
    >;
type OmittedOnly<Omitted, Aliased> = {
  [K in Exclude<Omitted, Aliased> & string]: OmittedValue;
} & { [K in Extract<Omitted, Aliased> & string]?: never };
type RenderObjects<Objects> = { [K in keyof Objects]: Render<Objects[K]> };
// Fields of a selection are not ordered as in the schema, nor as in expanded selections.
// The order does not matter to type relations, and visiting all fields of the schema type
// costs much more for large types.
type Flatten<Obj> = { [K in keyof Obj]: Obj[K] } & {};
// `any` (merged from fragments with errors) needs no special handling, as it absorbs the union
// of both branches.
type RenderBranch<B> = B extends AnyBranch
  ? Flatten<B[" $leaves"] & RenderObjects<B[" $objects"]> & OmittedFields<B>>
  : B;
/** Converts a selection, possibly in lists or nullable, into the type of its data. */
export type Render<T> = T extends readonly unknown[]
  ? Render<T[number]>[]
  : T extends null
    ? T
    : RenderBranch<T[keyof T]>;
