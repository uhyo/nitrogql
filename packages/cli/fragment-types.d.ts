// Utility types for merging fragments imported with `#import`, used by the content mapper.
//
// Selections that spread imported fragments are printed unrendered: each concrete object type
// in a selection is a `Branch` with its fields. Fields of fragments are merged into branches of
// the same object type, and `Render` converts the result into the same types as the ones printed
// for fully expanded selections.

/**
 * Selection of one concrete object type.
 * `Vars` has the values of boolean variables (used in `@skip` or `@include`) that the branch
 * is selected with, such as `{ flag: true }`.
 */
export type Branch<Name extends string, Orig, Fields, Aliased, Vars = {}> = {
  " $name": Name;
  " $orig": Orig;
  " $fields": Fields;
  " $aliased": Aliased;
  " $vars": Vars;
};
type AnyBranch = Branch<string, unknown, unknown, unknown, unknown>;
type IsAny<T> = 0 extends 1 & T ? true : false;

/**
 * Selection tree of an imported fragment, or `any` if the module does not provide it
 * (for example because it has errors).
 */
export type Fragment<
  Fragments,
  Name extends string,
> = Name extends keyof Fragments ? Fragments[Name] : any;

// Fields omitted by `@skip` or `@include` are optional `never`, which read as `undefined` (or `never`).
// Mapping over keys of both keeps a field optional only if it is omitted on both sides.
// Values are erased before intersecting, as conflicting values would reduce it to `never`.
type Keys<T> = { [K in keyof T]: unknown };
type MergeFields<A, B> = [keyof A & keyof B] extends [never]
  ? A & B
  : MergeOverlappingFields<A, B>;
type MergeOverlappingFields<A, B> = {
  [K in keyof (Keys<A> & Keys<B>)]: K extends keyof A
    ? K extends keyof B
      ? MergeValue<A[K], B[K]>
      : A[K]
    : K extends keyof B
      ? B[K]
      : never;
};
type MergeValue<A, B> = [A] extends [undefined]
  ? B
  : [B] extends [undefined]
    ? A
    : MergeMember<A, B>;
type MergeMember<A, B> = A extends AnyBranch
  ? MergeBranch<A, B>
  : A extends readonly (infer E)[]
    ? MergeValue<E, B extends readonly (infer F)[] ? F : never>[]
    : A;
type MergeBranch<
  A extends AnyBranch,
  B,
  Matching = Extract<B, MatchingBranch<A>>,
> = [Matching] extends [never] ? A : MergeMatchingBranches<A, Matching>;
// Branches of the same object type that agree on the values of the variables they share.
// Variables that one side does not have may have any value on the other.
// The index signature keeps the pattern from being a weak type, which would reject
// branches that share no variables.
type MatchingBranch<A extends AnyBranch> = {} extends A[" $vars"]
  ? { " $name": A[" $name"] }
  : {
      " $name": A[" $name"];
      " $vars": Partial<A[" $vars"]> & { [name: string]: boolean };
    };
// Distributes over branches of the same object type, which differ in variable values.
type MergeMatchingBranches<A extends AnyBranch, B> = B extends AnyBranch
  ? Branch<
      A[" $name"],
      A[" $orig"],
      MergeFields<A[" $fields"], B[" $fields"]>,
      MergeFields<A[" $aliased"], B[" $aliased"]>,
      A[" $vars"] & B[" $vars"]
    >
  : never;

/** Merges a fragment into a selection. */
export type MergeFragment<A, Fragment> =
  IsAny<Fragment> extends true
    ? any
    : A extends AnyBranch
      ? MergeBranch<A, Fragment>
      : A;
/**
 * Combines fragments spread in the same selection.
 * Unlike merging into a selection, branches of object types that only one fragment has are kept.
 */
export type CombineFragments<A, B> =
  IsAny<A> extends true
    ? any
    : IsAny<B> extends true
      ? any
      :
          | MergeMember<A, B>
          | Exclude<B, { " $name": A extends AnyBranch ? A[" $name"] : never }>;
/** Fragment spread that is skipped by `@skip` or `@include`. */
export type SkipFragment<Fragment> = Fragment extends AnyBranch
  ? Branch<
      Fragment[" $name"],
      Fragment[" $orig"],
      { [K in keyof Fragment[" $fields"]]?: never },
      { [K in keyof Fragment[" $aliased"]]?: never },
      Fragment[" $vars"]
    >
  : Fragment;

// Same as `__SelectionSet` in schema types.
type Beautify<Obj> = { [K in keyof Obj]: Obj[K] } & {};
type SelectionSet<Orig, Obj, Others> = Beautify<
  Pick<
    {
      [K in keyof Orig]: Obj extends { [P in K]?: infer V } ? V : unknown;
    },
    Extract<keyof Orig, keyof Obj>
  > &
    Others
>;
// Homomorphic, so that omitted fields stay optional.
type RenderFields<Fields> = { [K in keyof Fields]: Render<Fields[K]> };
/** Converts a selection into the type of its data. */
// `any` needs no special handling, as it absorbs the union of both branches.
export type Render<T> = T extends AnyBranch
  ? SelectionSet<
      T[" $orig"],
      RenderFields<T[" $fields"]>,
      RenderFields<T[" $aliased"]>
    >
  : T extends readonly (infer E)[]
    ? Render<E>[]
    : T;
