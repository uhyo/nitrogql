import { Highlight } from "@/app/_utils/Highlight";
import { ArticleMetadata } from "./_meta";
import Link from "next/link";

export const blogPostRelease1_3: ArticleMetadata = {
  slug: "release-1.3",
  title: "nitrogql 1.3 release: GraphQL Scalars support",
  shortDescription: `In nitrogql 1.3, we added support for integrating with GraphQL Scalars.
With a new built-in plugin, you can use GraphQL Scalars without manually defining custom scalar types.`,
  publishDate: new Date("2023-10-08T00:00Z"),
  render,
};

function render() {
  return (
    <>
      <p>
        Today, we are happy to announce release of <strong>nitrogql 1.3</strong>
        !
      </p>
      <p>
        <b>nitrogql</b> is a toolchain for using GraphQL in TypeScript projects.
        In 1.3, we added a new built-in plugin for integrating with{" "}
        <a href="https://the-guild.dev/graphql/scalars" target="_blank">
          GraphQL Scalars
        </a>
        .
      </p>

      <h3 id="integrating-with-graphql-scalars">
        Integrating with GraphQL Scalars
      </h3>
      <p>
        GraphQL Scalars is a library that provides a set of useful custom scalar
        types for GraphQL. Before 1.3, you could use GraphQL Scalars with
        nitrogql, but you had to manually define corresponding TypeScript types
        with the{" "}
        <Link href="/configuration/options#generate.type.scalarTypes">
          scalarTypes
        </Link>{" "}
        option.
      </p>
      <p>
        With the new built-in plugin,{" "}
        <code>nitrogql:graphql-scalars-plugin</code>, you can use GraphQL
        Scalars without manually defining custom scalar types. The plugin
        automatically recognizes suitable TypeScript types for GraphQL Scalars
        types.
      </p>

      <h3 id="usage">Usage</h3>
      <p>
        To use the plugin, you need to add it to the plugins array in the{" "}
        <Link href="/configuration/options">configuration file</Link>.
      </p>
      <Highlight language="yaml">
        {`schema:
  - ./schema/*.graphql
  - ./schema/scalars.ts
extensions:
  nitrogql:
    plugins:
      - "nitrogql:graphql-scalars-plugin"
    # ...`}
      </Highlight>

      <p>
        In the above example, <code>./schema/scalars.ts</code> is a TypeScript
        file that includes the GraphQL Scalars types. This file will look like:
      </p>
      <Highlight language="typescript">
        {`import { GraphQLSchema } from "graphql";
import { DateTimeResolver } from "graphql-scalars";

export default new GraphQLSchema({
  types: [DateTimeResolver],
});`}
      </Highlight>
      <p>
        With this configuration, nitrogql will automatically recognize{" "}
        <code>DateTime</code> as a scalar type. You can use it in your GraphQL
        schema without additional configuration.
      </p>
      <p>
        The exact work done by the plugin is to examine each{" "}
        <code>GraphQLScalarType</code> instance in the schema and find a
        corresponding TypeScript type in{" "}
        <code>extensions.codegenScalarType</code>. This data enables an
        automatic mapping from GraphQL Scalars types to TypeScript types by
        GraphQL toolings.
      </p>
      <p>
        This also means that other libraries that provide custom scalar types
        could also be supported by nitrogql as long as they provide a{" "}
        <code>GraphQLScalarType</code> instance with the{" "}
        <code>extensions.codegenScalarType</code> metadata. We are not aware of
        any other libraries that provide such metadata though.
      </p>

      <h3 id="conclusion">Conclusion</h3>

      <p>
        In this release, we improved support for GraphQL Scalars. With a new
        built-in plugin, you can use GraphQL Scalars without manually defining
        custom scalar types. Just include the GraphQL Scalars types in your{" "}
        <code>GraphQLSchema</code> object and that&apos;s it!
      </p>

      <p>
        Having a single source of truth for custom scalar types is always a good
        thing. It is good idea for libraries to provide such metadata so that
        the runtime implementation and the TypeScript type definition are always
        in sync. This release is a significant step towards that direction.
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
