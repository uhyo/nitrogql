import Link from "next/link";
import { Hint } from "@/app/_utils/Hint";
import { Highlight } from "@/app/_utils/Highlight";
import { Toc } from "../_toc";

export default function GettingStarted() {
  return (
    <Toc>
      <main>
        <h2>Configuration</h2>
        <p>
          nitrogql uses a configuration file to specify the location of your
          schema and operations, and to configure how types are generated. The
          configuration file should be placed in the root of your project.
        </p>
        <Hint>
          💡 The file format follows{" "}
          <a href="https://the-guild.dev/graphql/config/docs" target="_blank">
            the GraphQL Config convention
          </a>{" "}
          from The Guild. This enables you to share the configuration file with
          other GraphQL tools, if you use any.
        </Hint>
        <Hint>
          💡 Relative paths are always resolved from the location of the
          configuration file.
        </Hint>

        <h3 id="config-file-name">Default config file name</h3>
        <p>
          By default, configuration file is searched in the following order:
        </p>
        <ol>
          <li>
            <code>graphql.config.json</code>
          </li>
          <li>
            <code>graphql.config.yaml</code>
          </li>
          <li>
            <code>graphql.config.yml</code>
          </li>
          <li>
            <code>graphql.config.js</code>
          </li>
          <li>
            <code>graphql.config.mjs</code>
          </li>
          <li>
            <code>graphql.config.cjs</code>
          </li>
          <li>
            <code>.graphqlrc</code>
          </li>
          <li>
            <code>.graphqlrc.json</code>
          </li>
          <li>
            <code>.graphqlrc.yaml</code>
          </li>
          <li>
            <code>.graphqlrc.yml</code>
          </li>
          <li>
            <code>.graphqlrc.js</code>
          </li>
          <li>
            <code>.graphqlrc.mjs</code>
          </li>
          <li>
            <code>.graphqlrc.cjs</code>
          </li>
        </ol>

        <h4>Using JavaScript configuration files</h4>
        <p>
          When using JavaScript configuration files, a ES Module configuration
          file should default-export the configuration object. A CommonJS
          configuration file should assign the configuration object to{" "}
          <code>module.exports</code>.
        </p>
        <p>Also you can use JSDoc for type checking and auto-completion.</p>
        <Highlight language="js">
          {`/**
  * @type {import("@nitrogql/cli").NitrogqlConfig}
  */
const config = {
  schema: "./schema/*.graphql",
  documents: ["./app/**/*.graphql", "./common/**/*.graphql"],
  // ...
};

export default config;`}
        </Highlight>

        <h3 id="schema-operations">schema and operations</h3>
        <p>
          To specify the location of your schema and operations, use{" "}
          <code>schema</code> and <code>documents</code> top-level fields. These
          fields accept glob patterns and can be specified as a string or an
          array of strings.
        </p>
        <Highlight language="yaml">
          {`schema: "./schema/*.graphql"
documents:
  - "./app/**/*.graphql"
  - "./common/**/*.graphql"`}
        </Highlight>
        <p>
          Note that schema and operations are parsed with different parsers.
          Mixing them will cause errors.
        </p>
        <p>
          <strong>
            <code>schema</code> is always required.
          </strong>{" "}
          <code>documents</code> is optional. If you only have schema, you can
          still use nitrogql to check your schema.
        </p>
        <Hint>
          💡 Other configuration options are placed under{" "}
          <code>extensions.nitrogql</code> in the configuration file.
        </Hint>

        <h3 id="generate.schemaOutput">generate.schemaOutput</h3>
        <p>
          Where to output the generated schema types. Generated file is depended
          by generated operations types.
        </p>
        <p>
          <strong>
            <code>generate.schemaOutput</code> is required{" "}
          </strong>
          when you use the <code>generate</code> command.
        </p>
        <p>Example:</p>
        <Highlight language="yaml">
          {`schema: "./schema/*.graphql"
documents:
  - "./app/**/*.graphql"
  - "./common/**/*.graphql"
extensions:
  nitrogql:
    generate:
      schemaOutput: "./app/generated/schema.ts"`}
        </Highlight>

        <h3 id="generate.mode">generate.mode</h3>
        <p>
          Configures how types for operations are generated. Possible values
          are:
        </p>
        <ul>
          <li>
            <code>with-loader-ts-5.0</code> (default)
          </li>
          <li>
            <code>with-loader-ts-4.0</code>
          </li>
          <li>
            <code>standalone-ts-4.0</code>
          </li>
        </ul>
        <p>Example:</p>
        <Highlight language="yaml">
          {`schema: "./schema/*.graphql"
documents:
  - "./app/**/*.graphql"
  - "./common/**/*.graphql"
extensions:
  nitrogql:
    generate:
      mode: with-loader-ts-5.0
      schemaOutput: "./app/generated/schema.ts"`}
        </Highlight>

        <h4>with-loader-ts-5.0</h4>
        <p>
          Generates type definitions compatible with TypeScript 5.0 and above.
          This mode is recommended for projects using TypeScript 5.0 or above.
        </p>
        <p>
          This mode generates <code>foo.d.graphql.ts</code> next to{" "}
          <code>foo.graphql</code> which allows importing{" "}
          <code>foo.graphql</code> as a module.
        </p>
        <Hint>
          💡 With this mode, you need to configure <code>tsconfig.json</code> to
          enable the <code>allowArbitraryExtensions</code> compiler option.
        </Hint>
        <p>
          In order to import <code>.graphql</code> files as modules, you also
          need to configure your bundler to handle <code>.graphql</code> files.
          See <Link href="/getting-started">Getting Started</Link>.
        </p>

        <h4>with-loader-ts-4.0</h4>
        <p>Generates type definitions compatible with TypeScript 4.x.</p>
        <p>
          This mode generates <code>foo.graphql.d.ts</code> next to{" "}
          <code>foo.graphql</code> which allows importing{" "}
          <code>foo.graphql</code>
          as a module.
        </p>
        <p>
          In order to import <code>.graphql</code> files as modules, you also
          need to configure your bundler to handle <code>.graphql</code> files.
          See <Link href="/getting-started">Getting Started</Link>.
        </p>

        <h4>standalone-ts-4.0</h4>
        <p>
          Generates TypeScript coe compatible with TypeScript 4.x. This mode is
          recommended for projects which don&apos;t use bundlers.
        </p>
        <p>
          This mode generates <code>foo.graphql.ts</code> next to{" "}
          <code>foo.graphql</code> which allows importing{" "}
          <code>foo.graphql.ts</code> as a module. The generated code includes
          runtime code so you do not need to configure your bundler.
        </p>

        <h3 id="generate.schemaModuleSpecifier">
          generate.schemaModuleSpecifier
        </h3>
        <p>
          Configures what module specifier to use when importing the generated
          schema types from operations types. When set, all generated operations
          types will import the schema types from this exact module name. If not
          set, the generated operations types will import the schema types using
          relative paths.
        </p>
        <p>
          This option is especially useful in monorepo projects where you need
          to import the schema types from a different package.
        </p>
        <p>Example:</p>
        <Highlight language="yaml">
          {`schema: "./schema/*.graphql"
documents:
  - "./app/**/*.graphql"
  - "./common/**/*.graphql"
extensions:
  nitrogql:
    generate:
      schemaOutput: "./app/generated/schema.ts"
      schemaModuleSpecifier: "@/generated/schema"`}
        </Highlight>
        <p>
          With the above configuration, the generated operations types will
          import the schema types from <code>@/generated/schema</code> so they
          will look like:
        </p>
        <Highlight language="typescript">
          {`import * as Schema from "@/generated/schema";
// ...`}
        </Highlight>

        <h3 id="generate.scalarTypes">generate.scalarTypes</h3>
        <p>
          Configures how GraphQL scalar types are mapped to TypeScript types.
          The default mapping is:
        </p>
        <Highlight language="yaml">
          {`extensions:
  ID: string
  String: string
  Boolean: boolean
  Int: number
  Float: number`}
        </Highlight>
        <p>
          If you declare a custom scalar type in your schema, you must specify
          the mapping in the configuration file. Any TypeScript code is allowed
          as long as it is valid as a type.
        </p>
        <p>
          Mapping for built-in scalar types need not be specified unless you
          want to override them.
        </p>
        <p>Example:</p>
        <Highlight language="yaml">
          {`extensions:
  nitrogql:
    generate:
      scalarTypes:
        Date: Date`}
        </Highlight>

        <h3 id="generate.emitSchemaRuntime">generate.emitSchemaRuntime</h3>
        <p>
          If <code>true</code>, emit runtime code for generated schema types
          (one specified by <code>generate.schemaOutput</code>). Default is{" "}
          <code>false</code>.
        </p>
        <p>Currently, runtime code is emitted only for enums.</p>
        <Hint>
          ⚠️ If you set this option to <code>true</code>, the{" "}
          <code>schemaOutput</code> file cannot be a <code>.d.ts</code> file.
        </Hint>
        <p>Example:</p>
        <Highlight language="yaml">
          {`extensions:
  nitrogql:
    generate:
      schemaOutput: "./app/generated/schema.ts"
      emitSchemaRuntime: true`}
        </Highlight>
        <p>
          With the above configuration, the generated schema code will look
          like:
        </p>
        <Highlight language="typescript">
          {`// Always emitted for enums
export type UserType = "NormalUser" | "PremiumUser";
// Emitted only if emitSchemaRuntime is true
export const UserType = {
  NormalUser: "NormalUser",
  PremiumUser: "PremiumUser",
} as const;`}
        </Highlight>

        <h3 id="generate.name">generate.name</h3>
        <p>
          Set of configurations about names of generated variables and types.
        </p>
        <p>Default settings are:</p>
        <Highlight language="yaml">
          {`extensions:
  nitrogql:
    generate:
      name:
        # default values
        capitalizeOperationNames: true
        operationResultTypeSuffix: Result
        variablesTypeSuffix: Variables
        queryVariableSuffix: Query
        mutationVariableSuffix: Mutation
        subscriptionVariableSuffix: Subscription`}
        </Highlight>

        <h4 id="generate.name.capitalizeOperationNames">
          generate.name.capitalizeOperationNames
        </h4>
        <p>
          If <code>true</code>, capitalize the first letter of operation names.
          Default is <code>true</code>.
        </p>
        <p>
          This option can control how generated operation document can be
          imported via auto import feature of your editor. For example, if you
          have <code>query getUser</code> in your schema, it can be
          auto-imported by typing <code>GetUserQuery</code> in your code.
        </p>
        <Highlight language="typescript">
          {`import GetUserQuery from "./app/graphql/queries/getUser.graphql";`}
        </Highlight>
        <p>
          If you set <code>capitalizeOperationNames: false</code>, the generated
          operation document can be imported by typing <code>getUserQuery</code>{" "}
          instead.
        </p>

        <h4 id="generate.name.operationResultTypeSuffix">
          generate.name.operationResultTypeSuffix
        </h4>
        <p>
          Suffix of the operation result type. Default is{" "}
          <code>&quot;Result&quot;</code>.
        </p>
        <p>
          For example, if you have <code>query getUser</code> in your schema,
          the generated operation result type will be <code>GetUserResult</code>
          .
        </p>
        <Hint>
          💡 You can also use <code>ResultOf</code> from the{" "}
          <code>@graphql-typed-document-node/core</code> package to extract the
          result type from your operation document.
        </Hint>

        <h4 id="generate.name.variablesTypeSuffix">
          generate.name.variablesTypeSuffix
        </h4>
        <p>
          Suffix of the operation variables type. Default is{" "}
          <code>&quot;Variables&quot;</code>.
        </p>
        <p>
          For example, if you have <code>query getUser</code> in your schema,
          the generated operation variables type will be{" "}
          <code>GetUserVariables</code>.
        </p>
        <Hint>
          💡 You can also use <code>VariablesOf</code> from the{" "}
          <code>@graphql-typed-document-node/core</code> package to extract the
          variables type from your operation document.
        </Hint>

        <h4 id="generate.name.queryVariableSuffix">
          generate.name.queryVariableSuffix
        </h4>
        <p>
          Suffix of the query variable. Default is{" "}
          <code>&quot;Query&quot;</code>.
        </p>
        <p>
          For example, if you have <code>query getUser</code> in your schema,
          the generated query variable will be <code>GetUserQuery</code>.
        </p>

        <h4 id="generate.name.mutationVariableSuffix">
          generate.name.mutationVariableSuffix
        </h4>
        <p>
          Suffix of the mutation variable. Default is{" "}
          <code>&quot;Mutation&quot;</code>.
        </p>
        <p>
          For example, if you have <code>mutation createUser</code> in your
          schema, the generated mutation variable will be{" "}
          <code>CreateUserMutation</code>.
        </p>

        <h4 id="generate.name.subscriptionVariableSuffix">
          generate.name.subscriptionVariableSuffix
        </h4>
        <p>
          Suffix of the subscription variable. Default is{" "}
          <code>&quot;Subscription&quot;</code>.
        </p>
        <p>
          For example, if you have <code>subscription onUserCreated</code> in
          your schema, the generated subscription variable will be{" "}
          <code>OnUserCreatedSubscription</code>.
        </p>
      </main>
    </Toc>
  );
}
