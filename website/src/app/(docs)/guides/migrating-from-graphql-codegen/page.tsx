import { Hint } from "@/app/_utils/Hint";
import { Toc } from "../../_toc";
import { Breadcrumb } from "@/app/_utils/Breadcrumb";
import { Highlight } from "@/app/_utils/Highlight";
import Link from "next/link";

export default function Migrating() {
  return (
    <Toc>
      <main>
        <Breadcrumb
          parents={[{ label: "Guides", href: "/guides" }]}
          current="Migrating from GraphQL Code Generator"
        />
        <h2>Migrating from GraphQL Code Generator</h2>

        <p>
          <a href="https://graphql-code-generator.com/" target="_blank">
            GraphQL Code Generator
          </a>{" "}
          is a great tool for generating code from GraphQL schema and
          operations. Basically it has the same goal as nitrogql. This page
          guides you how to migrate from GraphQL Code Generator to nitrogql.
        </p>
        <p>
          GraphQL Code Generator has a couple of <b>presets</b> that control how
          TypeScript code is generated. nitrogql&apos;s approach is similar to
          the <b>near-operation-file</b> preset. This preset was the recommended
          preset while GraphQL Code Generator was in v2.
        </p>
        <p>
          While GraphQL Code Generator has changed their recommended preset to
          the <b>client</b> preset, nitrogql still endorses the idea of the{" "}
          <b>near-operation-file</b> preset.
        </p>

        <h3 id="prerequisites">Prerequisites</h3>
        <p>
          This guide assumes that you are using GraphQL Code Generator under the
          following conditions:
        </p>
        <ul>
          <li>
            You are using the <b>near-operation-file</b> preset.
          </li>
          <li>
            You write your GraphQL operations in <code>.graphql</code> files,
            not inside <code>.ts</code> files.
          </li>
        </ul>
        <p>
          If you diverge from these conditions, you need to first migrate to
          these conditions before migrating to nitrogql.
        </p>

        <Hint>
          😥 Generating resolver types is not supported in nitrogql yet. If you
          are using GraphQL Code Generator to generate resolver types, that part
          cannot be migrated to nitrogql for now.
        </Hint>

        <h3 id="before-migrating-to-nitrogql">Before migrating to nitrogql</h3>
        <p>
          Apart from the above fundamental differences, nitrogql has limited,
          opinionated set of configuration options. This means that some of the
          configuration options you used in GraphQL Code Generator may not be
          available in nitrogql.
        </p>
        <p>
          While you stay with GraphQL Code Generator, you first need to adjust
          its usage to be compatible with nitrogql.
        </p>

        <h4 id="use-typed-document-node">
          Use <code>TypedDocumentNode</code>
        </h4>
        <p>
          GraphQL Code Generator has a couple of plugins that generate
          TypeScript code from GraphQL operations. For example,{" "}
          <code>typescript-react-apollo</code> generates React Hooks for each
          GraphQL operation which use Apollo Client under the hood.
        </p>
        <p>
          However, nitrogql only supports <code>TypedDocumentNode</code>-based
          code generation. Don&apos;t worry, TypedDocumentNode can be used with
          any popular UI library or GraphQL client library. That&apos;s why
          GraphQL Code Generator also recommends using TypedDocumentNode.
        </p>
        <p>
          Therefore, you need to migrate to the{" "}
          <a
            href="https://the-guild.dev/graphql/codegen/plugins/typescript/typed-document-node"
            target="_blank"
          >
            <code>typed-document-node</code> plugin
          </a>
          . If you are not familiar with TypedDocumentNode,{" "}
          <a
            href="https://graphql.wtf/episodes/41-typed-document-node"
            target="_blank"
          >
            Episode #41 of graphql.wtf
          </a>{" "}
          is a great resource to learn how to migrate to{" "}
          <code>typed-document-node</code>.
        </p>

        <p>
          Before you can migrate to nitrogql, you need to be using only{" "}
          <code>typescript-operations</code> and{" "}
          <code>typed-document-node</code> plugins, not those library-specific
          ones.
        </p>

        <h4 id="disable-case-conversion">Disable case conversion</h4>
        <p>
          Under the default settings, GraphQL Code Generator converts
          identifiers to PascalCase. For example, <code>getUser</code> is
          converted to <code>GetUser</code> and <code>ENUM_VALUE</code> is
          converted to <code>EnumValue</code>.
        </p>
        <p>
          nitrogql does not do such case conversion by default. Therefore, the{" "}
          <a
            href="https://the-guild.dev/graphql/codegen/plugins/typescript/typescript-operations#namingconvention"
            target="_blank"
          >
            <code>namingConvention</code> option
          </a>{" "}
          of the <code>typescript-operations</code> plugin should be set to{" "}
          <code>keep</code>. If you change the <code>namingConvention</code>{" "}
          option, you may also need to change TypeScript code accordingly.
        </p>
        <Highlight language="yaml">
          {`# codegen.yml
config:
  namingConvention: keep`}
        </Highlight>

        <h4 id="set-enums-as-const">
          Set <code>enumsAsConst: true</code>
        </h4>
        <p>
          GraphQL Code Generator generates code from enums using
          TypeScript&apos;s <code>enum</code> syntax by default. However,
          nitrogql does not use that syntax. Instead, nitrogql uses plain union
          types.
        </p>
        <p>
          This difference is not a big deal, but it may cause some
          incompatibility issues. Therefore, it is recommended to set{" "}
          <a
            href="https://the-guild.dev/graphql/codegen/plugins/typescript/typescript#enumsasconst"
            target="_blank"
          >
            <code>enumsAsConst: true</code>
          </a>{" "}
          and solve any incompatibility issues before migrating to nitrogql.
        </p>
        <Highlight language="yaml">
          {`# codegen.yml
config:
  enumsAsConst: true`}
        </Highlight>

        <h4 id="change-output-extension">Change output extension</h4>
        <p>
          By default, the <b>near-operation-file</b> preset generates{" "}
          <code>foo.generated.ts</code> next to <code>foo.graphql</code>. This
          means that if you want to import code generated from{" "}
          <code>foo.graphql</code>, you need to import{" "}
          <code>foo.generated.ts</code>:
        </p>
        <Highlight language="typescript">
          {`// default setting of GraphQL Code Generator
import { fooQuery } from "./foo.generated";`}
        </Highlight>
        <p>
          On the other hand, nitrogql recommends to import directly from{" "}
          <code>foo.graphql</code>:
        </p>
        <Highlight language="typescript">
          {`// after migrating to nitrogql
import { fooQuery } from "./foo.graphql";`}
        </Highlight>
        <p>
          For the ease of migration, adjust GraphQL Code Generator&apos;s
          configuration to generate <code>foo.graphql.ts</code> instead of{" "}
          <code>foo.generated.ts</code>. This can be done by setting{" "}
          <code>extension: .graphql.ts</code> in{" "}
          <a
            href="https://the-guild.dev/graphql/codegen/plugins/presets/near-operation-file-preset#extension"
            target="_blank"
          >
            <code>presetConfig</code>
          </a>
          .
        </p>
        <p>
          After you change the extension, you need to update all import
          declarations to import from <code>.graphql</code> files instead of{" "}
          <code>.generated</code> files. Don&apos;t forget to update your
          <code>.gitignore</code> to ignore <code>.graphql.ts</code> files.
        </p>
        <Highlight language="yaml">
          {`# codegen.yml
generates:
  src/:
    # ...
    presetConfig:
      extension: .graphql.ts`}
        </Highlight>

        <h4 id="adjust-generated-type-names">Adjust generated type names</h4>
        <p>
          GraphQL Code Generator and nitrogql have different naming conventions
          for generated types. Before migrating to nitrogql, adjust your code to
          match nitrogql&apos;s naming convention.
        </p>
        <p>
          For example, when you have a query named <code>GetUser</code>, default
          output of GraphQL Code Generator and nitrogql are summarized as
          follows:
        </p>
        <table>
          <thead>
            <tr>
              <th />
              <th>GraphQL Code Generator</th>
              <th>nitrogql</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <th>operation document object</th>
              <td>
                <code>GetUserDocument</code>
              </td>
              <td>
                <code>GetUserQuery</code>
              </td>
            </tr>
            <tr>
              <th>operation result type</th>
              <td>
                <code>GetUserQuery</code>
              </td>
              <td>
                <code>GetUserResult</code>
              </td>
            </tr>
            <tr>
              <th>operation variables type</th>
              <td>
                <code>GetUserQueryVariables</code>
              </td>
              <td>
                <code>GetUserVariables</code>
              </td>
            </tr>
          </tbody>
        </table>
        <p>
          Note that <code>Query</code> in the table is substituted with{" "}
          <code>Mutation</code> or <code>Subscription</code> depending on the
          operation type.
        </p>
        <p>
          You can adjust the names of result type and variables type with the
          following settings:
        </p>
        <Highlight language="yaml">
          {`# GraphQL Code Generator config
config:
  omitOperationSuffix: true
  operationResultSuffix: Result
`}
        </Highlight>
        <p>
          As is the case with other configuration changes, you need to update
          all TypeScript code that imports these types.
        </p>
        <p>
          Name of the operation document object (<code>GetUserDocument</code>)
          still differ from nitrogql with the above setting. Since GraphQL Code
          Generator cannot exactly match the nitrogql behavior, we will guide
          you to configure nitrogql to match the current behavior of GraphQL
          Code Generator.
        </p>

        <h3 id="migrating-to-nitrogql">Migrating to nitrogql</h3>
        <p>Now you are ready to migrate to nitrogql!</p>

        <Hint>
          🤯 If you are in a monorepo setting, adjust below instructions as
          explained in the <Link href="/guides/monorepo">monorepo guide</Link>.
        </Hint>

        <h4 id="install-nitrogql">Install nitrogql</h4>
        <p>First, install nitrogql and its dependencies.</p>
        <Highlight language="bash">{`npm install -D @nitrogql/cli`}</Highlight>

        <p>
          If you are using webpack, you also need to install appropriate webpack
          loader. Note that this also applies to Next.js projects.
        </p>
        <Highlight language="bash">{`npm install -D @nitrogql/graphql-loader`}</Highlight>

        <p>
          If you are using Rollup, you need to install appropriate Rollup
          plugin. Note that this also applies to Vite projects.
        </p>
        <Highlight language="bash">{`npm install -D @nitrogql/rollup-plugin`}</Highlight>

        <h4 id="create-nitrogql-config">Create nitrogql config</h4>
        <p>
          Nitrogql&apos;s configuration file is either{" "}
          <code>graphql.config.yaml</code> or <code>.graphqlrc.yaml</code> at
          the root of your project. You might have already one depending on your
          GraphQL Code Generator configuration. Also, you can use{" "}
          <code>.json</code> or <code>.js</code> files instead of{" "}
          <code>.yaml</code> at your preference.
        </p>
        <p>
          You can reuse <code>schema</code> and <code>documents</code> options
          from your GraphQL Code Generator configuration. Start by copying them
          to your nitrogql configuration file:
        </p>
        <Highlight language="yaml">
          {`# graphql.config.yaml
schema: src/schema/*.graphql
documents: src/app/**/*.graphql
`}
        </Highlight>
        <p>
          Note that any other nitrogql options are put under{" "}
          <code>extensions.nitrogql</code> object.
        </p>

        <h4 id="configure-schema-output">Configure schema output</h4>
        <p>
          One option you need to set is <code>generate.schemaOutput</code>. This
          option controls where the generated schema type definition is written
          to. Set it to the path to the file where you want to write the schema
          type definition to. This option corresponds to the{" "}
          <a
            href="https://the-guild.dev/graphql/codegen/plugins/typescript/typescript"
            target="_blank"
          >
            <code>typescript</code> plugin
          </a>{" "}
          of GraphQL Code Generator.
        </p>
        <p>
          Also, if you are importing enums from the generated schema file, you
          need to set <code>generate.emitSchemaRuntime</code> to{" "}
          <code>true</code>. This is the default setting of GraphQL Code
          Generator, but nitrogql does not emit runtime enum definitions by
          default.
        </p>
        <p>
          <b>Example:</b>
        </p>
        <Highlight language="yaml">
          {`# GraphQL Code Generator configuration
generates:
  path/to/schema.ts:
    plugins:
      - typescript
    config:
      # ...

# corresponding nitrogql configuration (graphql.config.yaml)
schema: src/schema/*.graphql
documents: src/app/**/*.graphql
extensions:
  nitrogql:
    generate:
      schemaOutput: path/to/schema.ts
      emitSchemaRuntime: true`}
        </Highlight>

        <h4 id="configure-operation-output">Configure operation output</h4>
        <p>
          Next, you need to configure generation of TypeScript code from GraphQL
          operations. This corresponds to the{" "}
          <a
            href="https://the-guild.dev/graphql/codegen/plugins/typescript/typescript-operations"
            target="_blank"
          >
            <code>typescript-operations</code> plugin
          </a>{" "}
          of GraphQL Code Generator.
        </p>
        <p>
          Without additional configuration, nitrogql generates TypeScript code
          next to each GraphQL operations files. This is the same architecture
          as GraphQL Code Generator&apos;s <code>near-operation-file</code>{" "}
          preset.
        </p>
        <p>
          However, you need to adjust nitrogql&apos;s <code>generate</code>{" "}
          option so that you can use the generated code from your application in
          the same way as you did with GraphQL Code Generator.
        </p>
        <p>
          Below is the nitrogql configuration for keeping the same behavior as
          your current settings.
        </p>
        <Highlight language="yaml">
          {`schema: src/schema/*.graphql
documents: src/app/**/*.graphql
extensions:
  nitrogql:
    generate:
      schemaOutput: path/to/schema.ts
      emitSchemaRuntime: true
      # add below
      export:
        defaultExportForOperation: false
        variablesType: true
        operationResultType: true
      name:
        queryVariableSuffix: Document
        mutationVariableSuffix: Document
        subscriptionVariableSuffix: Document 
`}
        </Highlight>

        <h4 id="configure-typescript">Configure TypeScript</h4>
        <p>
          In order to use the generated code from your application, you might
          need to adjust TypeScript configuration to recognize the generated
          code.
        </p>
        <p>
          In your <code>tsconfig.json</code>, set the{" "}
          <code>allowArbitraryExtensions</code> compiler option to{" "}
          <code>true</code> so that TypeScript lets you import{" "}
          <code>.graphql</code> files.
        </p>
        <p>
          Note that this option is only available in TypeScript 5.0 or later. If
          you are using an older version of TypeScript, you can set
          nitrogql&apos;s{" "}
          <Link href="/configuration/options#generate.mode">
            <code>generate.mode</code>
          </Link>{" "}
          option to <code>with-loader-ts-4.0</code>.
        </p>

        <h4 id="configure-webpack-loader-or-rollup-plugin">
          Configure webpack loader or Rollup plugin
        </h4>
        <p>
          As the last step, you need to configure webpack loader or Rollup
          plugin so that they can load <code>.graphql</code> files.
        </p>
        <p>
          If you are using webpack, add the following to your webpack
          configuration:
        </p>
        <Highlight language="javascript">
          {`// webpack.config.js
module.exports = {
  module: {
    rules: [
      // ...
      {
        test: /\\.graphql$/,
        use: [
          {
            loader: "@nitrogql/graphql-loader",
            options: {
              // path to your nitrogql configuration file
              configFile: "./graphql.config.yaml",
            },
          },
        ],
      },
    ],
  },
};`}
        </Highlight>
        <p>
          If you are using Rollup, add the following to your Rollup
          configuration:
        </p>
        <Highlight language="javascript">
          {`// rollup.config.js
import graphql from "@nitrogql/rollup-plugin";

export default {
  // ...
  plugins: [
    // ...
    graphql({
      // path to your nitrogql configuration file
      configFile: "./graphql.config.yaml",
      include: ["**/*.graphql"],
    }),
  ],
};`}
        </Highlight>

        <h3 id="using-nitrogql-cli">Using nitrogql CLI</h3>
        <p>
          After you migrate to nitrogql, you need to also migrate build scripts
          to use nitrogql CLI.
        </p>
        <p>
          Basically, you need to replace <code>graphql-codegen</code> command
          with <code>nitrogql generate</code>.
        </p>

        <h4 id="watch-mode">Watch mode</h4>
        <p>
          nitrogql CLI does not have a watch mode for now. If you need a watch
          mode, you can use{" "}
          <a href="https://nodemon.io/" target="_blank">
            nodemon
          </a>{" "}
          or{" "}
          <a
            href="https://github.com/open-cli-tools/chokidar-cli"
            target="_blank"
          >
            chokidar-cli
          </a>{" "}
          to watch GraphQL files and run <code>nitrogql generate</code>{" "}
          automatically.
        </p>
        <p>
          For example, if you are using chokidar-cli, a command for watching
          GraphQL files and running <code>nitrogql generate</code> is as
          follows:
        </p>
        <Highlight language="bash">
          {`chokidar '**/*.graphql' --initial --command 'npx nitrogql generate'`}
        </Highlight>

        <hr />

        <Hint>
          🧺 <b>Read Next</b>: <Link href="/configuration">Configuration</Link>,{" "}
          <Link href="/cli">CLI Usage</Link>
        </Hint>
      </main>
    </Toc>
  );
}
