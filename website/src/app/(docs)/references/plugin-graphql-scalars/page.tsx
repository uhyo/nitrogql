import Link from "next/link";
import { Highlight } from "@/app/_utils/Highlight";
import { Toc } from "../../_toc";
import { Breadcrumb } from "@/app/_utils/Breadcrumb";
import { ogp } from "@/app/_utils/metadata";
import { Hint } from "@/app/_utils/Hint";

export const metadata = ogp({
  title: "nitrogql:graphql-scalars plugin",
});

export default function GraphqlScalarsPlugin() {
  return (
    <Toc>
      <main>
        <Breadcrumb
          parents={[{ label: "References", href: "/references" }]}
          current="nitrogql:graphql-scalars plugin"
        />
        <h2>
          <code>nitrogql:graphql-scalars</code> plugin
        </h2>
        <p>
          <code>nitrogql:graphql-scalars-plugin</code> is a built-in plugin for
          integrating nicely with{" "}
          <a href="https://the-guild.dev/graphql/scalars" target="_blank">
            GraphQL Scalars
          </a>
          . GraphQL Scalars is a library that provides a set of useful custom
          scalar types for GraphQL.
        </p>
        <p>
          With this plugin, nitrogql automatically recognizes suitable
          TypeScript types for GraphQL Scalars types, reducing the need for
          manually defining custom scalars types with the{" "}
          <Link href="/configuration/options#generate.type.scalarTypes">
            scalarTypes
          </Link>{" "}
          option.
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

        <h3 id="using-graphql-scalars">Using GraphQL Scalars</h3>
        <p>
          In order for this plugin to work, you need to use a TypeScript file to
          include the GraphQL Scalars types. Above example demonstrates this
          with the <code>./schema/scalars.ts</code> file.
        </p>
        <p>
          The <code>./schema/scalars.ts</code> should default-export a{" "}
          <code>GraphQLSchema</code> instance that includes the GraphQL Scalars
          types. For example, if you want to use the <code>DateTime</code>{" "}
          scalar type, you can do the following:
        </p>
        <Highlight language="typescript">
          {`import { GraphQLSchema } from "graphql";
import { DateTimeResolver } from "graphql-scalars";

export default new GraphQLSchema({
  types: [DateTimeResolver],
});`}
        </Highlight>
        <Hint>
          🧸 <b>Tip:</b> in spite of the name, <code>DateTimeResolvers</code> is
          an instance of <code>GraphQLScalarType</code>. This object indeed
          includes the resolver functions for the <code>DateTime</code> scalar
          type, but also includes other metadata required for nitrogql to
          recognize the type.
        </Hint>

        <p>
          With this setting, no additional steps are required to use the{" "}
          <code>DateTime</code> scalar type in your GraphQL schema. This means
          that you don&apos;t need to define a custom scalar type with the{" "}
          <code>scalar</code> syntax in SDL:
        </p>
        <Highlight language="graphql">
          {`# This is *not* required
scalar DateTime`}
        </Highlight>

        <p>
          The corresponding TypeScript type for the <code>DateTime</code> scalar
          type is included in <code>DateTimeResolver</code>. This information
          will be used by nitrogql to generate the correct TypeScript types for
          your schema. Therefore, no additional configuration is required for
          the <code>scalarTypes</code> option.
        </p>
        <Highlight language="yaml">
          {`extensions:
  nitrogql:
    generate:
      # ...
      type:
        scalarTypes:
          # This is *not* required
          DateTime: Date`}
        </Highlight>
      </main>
    </Toc>
  );
}
