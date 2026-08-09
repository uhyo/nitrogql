import { Highlight } from "@/app/_utils/Highlight";
import { ArticleMetadata } from "./_meta";

export const blogPostRelease2_0: ArticleMetadata = {
  slug: "release-2.0",
  title: "nitrogql 2.0 release: @oneOf, operation descriptions, Node.js 22+",
  shortDescription: `nitrogql 2.0 adds support for the @oneOf directive and for
descriptions on operations, fragments and variable definitions.
As this release drops support for Node.js 18 and 20, it is a major version bump.
`,
  publishDate: new Date("2026-08-09T00:00Z"),
  render,
};

function render() {
  return (
    <>
      <p>
        Today, we are happy to announce release of <strong>nitrogql 2.0</strong>
        !
      </p>
      <p>
        <b>nitrogql</b> is a toolchain for using GraphQL in TypeScript projects.
        In 2.0, we added support for recent additions to the GraphQL
        specification: the <code>@oneOf</code> directive, and descriptions on
        operations, fragments and variable definitions. Also, this release drops
        support for Node.js versions that reached their end of life, which is
        why this is a major version bump.
      </p>

      <h2 id="nodejs-22-required">Node.js 22+ is now required</h2>
      <p>
        nitrogql packages now require <strong>Node.js 22 or later</strong>.
        Node.js 18 and 20 have both reached their end of life, and dropping them
        allows us to keep the codebase modern. This is the only breaking change
        in this release; if you are already on Node.js 22 or later, upgrading to
        nitrogql 2.0 should require no changes to your project.
      </p>
      <p>
        Published packages now declare this requirement through the{" "}
        <code>engines</code> field, so npm will warn you if you install them on
        an older Node.js version. Relatedly,{" "}
        <code>@nitrogql/esbuild-register</code> is now distributed as an ES
        module.
      </p>

      <h2 id="oneof">Support for the @oneOf directive</h2>
      <p>
        The <code>@oneOf</code> directive is a recent addition to the GraphQL
        specification that lets you define a polymorphic input object: an input
        object in which <em>exactly one</em> of the fields must be provided.
      </p>
      <Highlight language="graphql">
        {`input UserBy @oneOf {
  id: ID
  email: String
}`}
      </Highlight>
      <p>
        nitrogql 2.0 fully understands such input objects. The static check
        validates that literal values and variables passed to a{" "}
        <code>@oneOf</code> input object provide exactly one field with a
        non-null value. And notably, the generated TypeScript type is a union
        that lets the type checker enforce the same constraint at the TypeScript
        level:
      </p>
      <Highlight language="typescript">
        {`export type UserBy = {
  readonly id: ID;
  readonly email?: never;
} | {
  readonly id?: never;
  readonly email: String;
};`}
      </Highlight>
      <p>
        With this type, providing both fields, or providing none of them, is a
        type error in your TypeScript code. This is another example of
        nitrogql&apos;s goal of maximizing type safety of GraphQL operations.
      </p>

      <h2 id="operation-descriptions">
        Descriptions on operations, fragments and variable definitions
      </h2>
      <p>
        The latest edition of the GraphQL specification allows descriptions to
        be attached to executable definitions (operations and fragments) and to
        variable definitions. nitrogql 2.0 supports parsing these descriptions:
      </p>
      <Highlight language="graphql">
        {`"""
Query to get the current user.
"""
query getUser(
  "ID of the user."
  $id: ID!
) {
  user(id: $id) {
    name
  }
}`}
      </Highlight>
      <p>
        These descriptions are not just parsed; they are surfaced as JSDoc
        comments in the generated TypeScript code. Operation descriptions are
        attached to the generated result type and operation constant, fragment
        descriptions to the fragment type and constant, and variable
        descriptions to the corresponding properties of the variables type. This
        means that the descriptions you write in your GraphQL files will show up
        in your editor when you use the generated types.
      </p>
      <p>
        Note that descriptions are intentionally not included in the runtime
        document (the <code>TypedDocumentNode</code> JSON), so the generated
        runtime output remains compatible with consumers that do not know about
        the new syntax.
      </p>

      <h2 id="bug-fixes">Bug fixes</h2>
      <p>This release also contains several bug fixes:</p>
      <ul>
        <li>Fixed a parser panic on the shorthand (anonymous) query syntax.</li>
        <li>
          Fixed the GraphQL printer dropping default values and directives from
          variable definitions.
        </li>
        <li>
          Fixed handling of preopened directories in the WASI runtime shipped
          with <code>@nitrogql/wasi-preview1</code>.
        </li>
      </ul>

      <h2 id="conclusion">Conclusion</h2>
      <p>
        nitrogql 2.0 keeps up with the evolution of the GraphQL specification by
        supporting the <code>@oneOf</code> directive and descriptions on
        executable definitions, both of which are reflected in the generated
        TypeScript types. While this release is a major version bump, the only
        breaking change is the drop of old Node.js versions, so we expect the
        upgrade to be smooth for most projects.
      </p>
      <hr />

      <p>
        <em>
          nitrogql is developed by{" "}
          <a href="https://x.com/uhyo_" target="_blank">
            uhyo
          </a>
          . Contribution is more than welcome!
        </em>
      </p>
    </>
  );
}
