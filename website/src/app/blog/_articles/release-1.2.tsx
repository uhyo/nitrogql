import { Highlight } from "@/app/_utils/Highlight";
import { ArticleMetadata } from "./_meta";
import Link from "next/link";
import { Hint } from "@/app/_utils/Hint";

export const blogPostRelease1_2: ArticleMetadata = {
  slug: "release-1.2",
  title: "nitrogql 1.2 release: TypeScript support for config file, and more",
  shortDescription: `In nitrogql 1.2, we added support for using TypeScript for the configuration file.
Also, schema files can be in TypeScript.`,
  publishDate: new Date("2023-09-30T00:00Z"),
  render,
};

function render() {
  return (
    <>
      <p>
        Today, we are happy to announce release of <strong>nitrogql 1.2</strong>
        !
      </p>
      <p>
        <b>nitrogql</b> is a toolchain for using GraphQL in TypeScript projects.
        In 1.2, we added support for config files written in TypeScript (
        <code>graphql.config.ts</code>
        ). Also, using TypeScript for schema files is now supported.
      </p>

      <h3 id="config-file-in-typescript">Config file in TypeScript</h3>
      <p>
        Before 1.2, the config file for nitrogql could be written in YAML, JSON
        or JavaScript. In 1.2, TypeScript is supported as well. With help of
        TypeScript, you can easily get type safety and editor support for
        writing config file.
      </p>
      <Highlight language="typescript">
        {`// graphql.config.ts
import type { NitrogqlConfig } from "@nitrogql/cli";

const config: NitrogqlConfig = {
  schema: "./src/schema/*.graphql",
  // ...
};
export default config;
`}
      </Highlight>
      <p>
        This feature is powered by{" "}
        <a href="https://esbuild.github.io/" target="_blank">
          esbuild
        </a>{" "}
        and{" "}
        <a href="https://github.com/egoist/esbuild-register" target="_blank">
          esbuild-register
        </a>
        .
      </p>

      <h3 id="schema-file-in-typescript">Schema file in TypeScript</h3>
      <p>
        A GraphQL schema tends to be written in GraphQL Schema Definition
        Language (SDL) as a <code>.graphql</code> file. However, in some cases,
        you may want to use TypeScript for defining schema. To support this,
        nitrogql 1.2 added support for using TS files (or JS files, if you like)
        as schema files.
      </p>
      <p>
        A TypeScript schema file should default-export a string containing SDL:
      </p>
      <Highlight language="typescript">
        {`// schema.ts
export default \`
  scalar Date
  # ...
\`;
`}
      </Highlight>
      <p>
        Or if you have a{" "}
        <a
          href="https://graphql.org/graphql-js/type/#graphqlschema"
          target="_blank"
        >
          GraphQLSchema
        </a>{" "}
        object, you can export it as well:
      </p>
      <Highlight language="typescript">
        {`// schema.ts
import { GraphQLSchema } from "graphql";

const schema = new GraphQLSchema({ /* ... */ });
export default schema;
`}
      </Highlight>

      <p>
        Note, although esbuild is known to be extremely fast, loading TypeScript
        files requires a Node.js process to be spawned, which is not as fast as
        loading GraphQL files. So, if you care about performance, you may
        consider keep usage of TypeScript to a minimum.
      </p>

      <p>
        Also, currently, using TypeScript for schema files does not affect the
        type definition generation.
      </p>

      <h3 id="plan-for-supporting-graphql-scalars">
        Plan for supporting GraphQL Scalars
      </h3>
      <p>
        We have received a request for a nice{" "}
        <a href="https://the-guild.dev/graphql/scalars" target="_blank">
          GraphQL Scalars
        </a>{" "}
        integration.
      </p>
      <p>
        The ability to use TypeScript for schema files is a step towards this
        goal. Now you can write TypeScript code to import type definitions from{" "}
        <code>graphql-scalars</code> and export them as a schema so that
        nitrogql can recognize them.
      </p>
      <Highlight language="typescript">
        {`// schema.ts
import { typeDefs } from "graphql-scalars";

export default typeDefs.join("\\n");
`}
      </Highlight>
      <p>
        However, this does not automatically add corresponding TypeScript
        definitions to the generated type definitions, which means that you
        still need to manually add them to your configuration (
        <Link href="/configuration/options#generate.type.scalarTypes">
          generate.type.scalarTypes
        </Link>
        ).
      </p>
      <p>
        We want to make this process easier. We are evaluating several
        approaches to achieve this goal. If you have any idea, please let us
        know!
      </p>

      <h3 id="conclusion">Conclusion</h3>
      <p>
        In this release, we added support for using TypeScript for config files
        and schema files. We hope you enjoy this release!
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
