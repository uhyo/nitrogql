// Fragments with errors export `any` as their selections, and `Fragment` gives `any` for
// fragments that a module does not provide. Expanded types make the whole definition `any`
// instead, so these are not compared with them; this checks that merging them gives `any` for
// the branches they are merged into, whichever side of a merge they are on.
import type * as N from "./generated/fragment-types";

type IsAny<T> = 0 extends 1 & T ? true : false;
type Expect<T extends true> = T;

type Post = { Post: N.Branch<{ id: string }, {}, never> };
type User<Objects> = {
  User: N.Branch<{ name: string }, Objects, never>;
};
type RenderUser<Branches> = N.Render<{ User: Branches }>;

export type _0 = Expect<IsAny<RenderUser<N.Spread<N.At<any, "User">>>>>;
export type _1 = Expect<
  IsAny<RenderUser<N.MergeLeaves<N.At<any, "User">, { id: string }, never>>>
>;
export type _2 = Expect<
  IsAny<
    RenderUser<
      N.MergeFragment<N.Branch<{ id: string }, { posts: Post[] }, never>, any>
    >
  >
>;
export type _3 = Expect<
  IsAny<RenderUser<N.CombineFragments<N.At<User<{}>, "User">, any>>>
>;
export type _4 = Expect<IsAny<RenderUser<N.SkipFragment<any>>>>;
// A fragment whose `posts` merged a fragment with errors, combined with one that selects
// `posts` as well, on either side.
type WithError = User<{ posts: { Post: any }[] }>;
type WithoutError = User<{ posts: Post[] }>;
export type _5 = Expect<
  IsAny<
    RenderUser<
      N.CombineFragments<N.At<WithError, "User">, N.At<WithoutError, "User">>
    >["posts"][number]
  >
>;
export type _6 = Expect<
  IsAny<
    RenderUser<
      N.CombineFragments<N.At<WithoutError, "User">, N.At<WithError, "User">>
    >["posts"][number]
  >
>;
