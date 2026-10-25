import { Highlight } from "@/app/_utils/Highlight";
import { Toc } from "../../_toc";
import { Breadcrumb } from "@/app/_utils/Breadcrumb";
import { Hint } from "@/app/_utils/Hint";
import Link from "next/link";
import { ogp } from "@/app/_utils/metadata";

export const metadata = ogp({
  title: "Using TypeScript Content Mapper",
});

export default function TypeScriptContentMapper() {
  return (
    <Toc>
      <main>
        <Breadcrumb
          parents={[{ label: "Guides", href: "/guides" }]}
          current="Using TypeScript Content Mapper"
        />
        <h2>Using TypeScript Content Mapper</h2>

        <p>
          TypeScript 7.1 introduced <strong>content mappers</strong>, a plugin
          mechanism that lets TypeScript understand files other than TypeScript
          and JavaScript. nitrogql provides a content mapper for{" "}
          <code>.graphql</code> files. With it, TypeScript reads your GraphQL
          files directly and you no longer need to run{" "}
          <code>nitrogql generate</code> to get types for your operations.
        </p>
        <Hint>
          🧪 Content mappers are a new feature of TypeScript 7.1, which is
          currently in beta. nitrogql&apos;s support for content mappers is
          experimental too.
        </Hint>

        <h3 id="how-it-works">How It Works</h3>
        <p>
          Usually, <code>nitrogql generate</code> generates a{" "}
          <code>.d.graphql.ts</code> file next to each <code>.graphql</code>{" "}
          file, and TypeScript reads the generated file when you import a{" "}
          <code>.graphql</code> file.
        </p>
        <p>
          With the content mapper, TypeScript asks nitrogql for the types of a{" "}
          <code>.graphql</code> file whenever it needs them. Nothing is written
          to the disk, so there is no generated file to keep up to date. Since
          the types always reflect the current content of your GraphQL files,
          editors show the result of your edits immediately.
        </p>
        <p>
          Additionally, errors in your GraphQL files (syntax errors and
          validation errors against the schema) are reported by TypeScript as
          well. Each error has a code such as <code>nitrogql2001</code>; see{" "}
          <Link href="/references/error-codes">Error codes</Link> for the list.
        </p>
        <p>
          While a GraphQL file has errors, the operations and fragments it
          exports are typed as <code>any</code>, so that code importing the file
          does not get unrelated errors.
        </p>

        <h3 id="setup">Setup</h3>
        <p>
          First, make sure you are using TypeScript 7.1 or later, and that{" "}
          <code>@nitrogql/cli</code> is installed.
        </p>
        <Highlight language="bash">
          {`npm install --save-dev typescript@beta @nitrogql/cli`}
        </Highlight>
        <p>
          Then, register nitrogql&apos;s content mapper in your{" "}
          <code>tsconfig.json</code>:
        </p>
        <Highlight language="json">
          {`{
  "compilerOptions": {
    // ...
  },
  "contentMappers": [
    {
      "package": "@nitrogql/cli",
      "extensions": [".graphql"]
    }
  ]
}`}
        </Highlight>
        <p>
          Your nitrogql configuration file is used as usual. By default, the
          content mapper looks for a configuration file in the same directory as{" "}
          <code>tsconfig.json</code>. Note that the content mapper only needs
          the <code>schema</code> and <code>documents</code> options; you can
          remove the <code>schemaOutput</code> option if you only used it for
          operation types.
        </p>
        <Highlight language="yaml">
          {`schema: ./schema/*.graphql
documents: ./app/**/*.graphql
`}
        </Highlight>
        <p>
          Content mappers are external programs executed by TypeScript, so you
          need to explicitly allow running them. On the command line, pass the{" "}
          <code>--runExternalCode</code> flag to <code>tsc</code>:
        </p>
        <Highlight language="bash">{`npx tsc --runExternalCode`}</Highlight>
        <p>
          In VS Code, content mappers run when the workspace is trusted and the{" "}
          <code>js/ts.contentMappers.enabled</code> setting is enabled.
        </p>

        <h3 id="schema-types">Schema Types</h3>
        <p>
          Types for operations depend on types generated from your schema. When
          your schema consists of GraphQL files, the content mapper provides
          schema types too, and no generated files are needed at all. Schema
          types are provided as the types of the first schema file, so you can
          even import schema types from that file:
        </p>
        <Highlight language="typescript">
          {`import type { User } from "./schema/schema.graphql";`}
        </Highlight>
        <p>
          If your schema files have a different extension (such as{" "}
          <code>.graphqls</code>), add the extension to <code>extensions</code>{" "}
          in <code>tsconfig.json</code> as well.
        </p>
        <p>
          When the <code>schemaOutput</code> option or the{" "}
          <code>schemaModuleSpecifier</code> option is set, operation types
          import schema types from there instead. This is required when your
          schema is given as an introspection JSON or as a{" "}
          <Link href="/configuration/options#schema-file-types">
            JavaScript/TypeScript schema file
          </Link>
          , which TypeScript cannot read through the content mapper. In that
          case, keep running <code>nitrogql generate</code> to generate the
          schema type definition file.
        </p>

        <h3 id="options">Options</h3>
        <p>
          The content mapper accepts options via the <code>options</code> field
          in <code>tsconfig.json</code>.
        </p>
        <h4 id="option-config-file">
          <code>configFile</code>
        </h4>
        <p>
          Path to the nitrogql configuration file, relative to{" "}
          <code>tsconfig.json</code>. Use this when your configuration file is
          not in the same directory as <code>tsconfig.json</code>, or has a
          non-default name.
        </p>
        <Highlight language="json">
          {`"contentMappers": [
  {
    "package": "@nitrogql/cli",
    "extensions": [".graphql"],
    "options": {
      "configFile": "./config/graphql.config.ts"
    }
  }
]`}
        </Highlight>

        <h3 id="notes">Notes</h3>
        <ul>
          <li>
            The content mapper only provides types. You still need a{" "}
            <Link href="/guides/getting-started">loader</Link> for your bundler
            so that <code>.graphql</code> files work at runtime.
          </li>
          <li>
            Generated files for GraphQL servers (<code>resolversOutput</code>{" "}
            and <code>serverGraphqlOutput</code>) are still generated by{" "}
            <code>nitrogql generate</code>.
          </li>
          <li>
            After migrating, you can delete previously generated{" "}
            <code>.d.graphql.ts</code> files. The{" "}
            <code>allowArbitraryExtensions</code> compiler option is not needed
            for the content mapper.
          </li>
          <li>
            When TypeScript reports GraphQL errors, it skips reporting type
            errors in TypeScript code until those are fixed, as it does for
            syntax errors.
          </li>
        </ul>
      </main>
    </Toc>
  );
}
