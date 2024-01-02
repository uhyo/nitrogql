import Link from "next/link";
import { Hint } from "@/app/_utils/Hint";
import { Highlight } from "@/app/_utils/Highlight";
import { Toc } from "../../_toc";
import { Breadcrumb } from "@/app/_utils/Breadcrumb";
import { ogp } from "@/app/_utils/metadata";
import { InPageNav } from "@/app/_utils/InPageNav";

export const metadata = ogp({
  title: "Configuration Options",
});

export default function ConfigurationOptions() {
  return (
    <Toc>
      <main>
        <Breadcrumb
          parents={[{ label: "Configuration", href: "/configuration" }]}
          current="Configuration Options"
        />
        <h2>Configuration Options</h2>
        <p>
          This page describes all the configuration options available in the
          configuration file.
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

        <h4 id="schema-file-types">Schema file types</h4>
        <p>Nitrogql supports three types of schema files:</p>
        <ul>
          <li>
            <code>.graphql</code> files (GraphQL SDL)
          </li>
          <li>
            <code>.json</code> files (introspection result)
          </li>
          <li>
            <code>.js</code>/<code>.ts</code> files
          </li>
        </ul>
        <p>
          Nitrogql automatically detects the type of the schema file based on
          the file extension.
        </p>
        <p>
          A <code>.js</code>/<code>.ts</code> schema file must default-export
          schema either as a string or as a{" "}
          <a
            href="https://graphql.org/graphql-js/type/#graphqlschema"
            target="_blank"
          >
            GraphQLSchema
          </a>{" "}
          object.
        </p>

        <h4 id="operation-file-types">Operation file type</h4>
        <p>
          Nitrogql only supports <code>.graphql</code> files for operations.
        </p>

        <Hint>
          💡 Other configuration options are placed under{" "}
          <code>extensions.nitrogql</code> in the configuration file.
        </Hint>

        <h3 id="plugins">plugins</h3>
        <p>
          The <code>plugins</code> field is used to configure which plugins to
          use.
        </p>
        <p>
          Currently, third-party plugins are not supported. You can only use
          built-in plugins. Available plugins are:
        </p>
        <ul>
          <li>
            <code>nitrogql:model-plugin</code>
            <code>nitrogql:graphql-scalars-plugin</code>
          </li>
        </ul>
        <p>Example:</p>
        <Highlight language="yaml">
          {`extensions:
  nitrogql:
    plugins:
      - nitrogql:model-plugin`}
        </Highlight>

        <h3 id="generate.schemaOutput">generate.schemaOutput</h3>
        <p>
          Where to output the generated schema types. Generated file is depended
          by generated operations types.
        </p>
        <p>
          If you do not specify <code>generate.schemaOutput</code>:
        </p>
        <ul>
          <li>
            You can still use the <code>check</code> command to check your
            schema.
          </li>
          <li>
            The <code>generate</code> command will not generate the schema
            types. To use the <code>generate</code> command without specifying{" "}
            <code>generate.schemaOutput</code>, you need to specify{" "}
            <a href="#generate.schemaModuleSpecifier">
              generate.schemaModuleSpecifier
            </a>{" "}
            so that generated operations types know where to import the schema
            types from.
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
      schemaOutput: "./app/generated/schema.ts"`}
        </Highlight>

        <h3 id="generate.serverGraphqlOutput">generate.serverGraphqlOutput</h3>
        <p>
          When set, the <code>generate</code> command will generate a single
          TypeScript file which contains the entire GraphQL schema. This file
          can be used to create a GraphQL server.
        </p>
        <p>Example:</p>
        <Highlight language="yaml">
          {`schema: "./schema/*.graphql"
documents:
  - "./app/**/*.graphql"
extensions:
  nitrogql:
    generate:
      serverGraphqlOutput: "./app/generated/graphql.ts"`}
        </Highlight>
        <p>With the above configuration, the generated code will look like:</p>
        <Highlight language="typescript">
          {`// ./app/generated/graphql.ts
export const schema = \`
scalar String
scalar Boolean
# ...
type Query {
  # ...
}
# ...
\`;
`}
        </Highlight>

        <h3 id="generate.resolversOutput">generate.resolversOutput</h3>
        <p>
          When set, the <code>generate</code> command will generate a single
          TypeScript file which contains type definitions for resolvers. This is
          helpful for writing resolvers in a type-safe manner.
        </p>
        <p>
          This file depends on the generated schema types. Therefore, you need
          to configure either <code>generate.schemaOutput</code> or{" "}
          <code>generate.schemaModuleSpecifier</code> to use this option.
        </p>
        <p>Example:</p>
        <Highlight language="yaml">
          {`schema: "./schema/*.graphql"
extensions:
  nitrogql:
    generate:
      resolversOutput: "./app/generated/resolvers.ts"`}
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
          See <Link href="/guides/getting-started">Getting Started</Link>.
        </p>

        <h4>with-loader-ts-4.0</h4>
        <p>Generates type definitions compatible with TypeScript 4.x.</p>
        <p>
          This mode generates <code>foo.graphql.d.ts</code> next to{" "}
          <code>foo.graphql</code> which allows importing{" "}
          <code>foo.graphql</code> as a module.
        </p>
        <p>
          In order to import <code>.graphql</code> files as modules, you also
          need to configure your bundler to handle <code>.graphql</code> files.
          See <Link href="/guides/getting-started">Getting Started</Link>.
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
        <p>
          Note that you also need to configure your bundler to resolve{" "}
          <code>@/generated/schema</code> correctly (to{" "}
          <code>app/generated/schema.ts</code>).
        </p>

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

        <h3 id="generate.type">generate.type</h3>
        <p>Set of configurations about details of generated types.</p>
        <p>Default settings are:</p>
        <Highlight language="yaml">
          {`extensions:
  nitrogql:
    generate:
      type:
        # default values
        scalarTypes: {}
        allowUndefinedAsOptionalInput: true`}
        </Highlight>

        <h4 id="generate.type.scalarTypes">scalarTypes</h4>
        <p>
          Configures how GraphQL scalar types are mapped to TypeScript types.
          The default mapping is:
        </p>
        <Highlight language="yaml">
          {`scalarTypes:
  ID:
    send: string | number
    receive: string
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
        <Hint>
          💡 If you are using the{" "}
          <Link href="/references/plugin-graphql-scalars">
            nitrogql:graphql-scalars-plugin
          </Link>
          , you do not need to specify the mapping for GraphQL Scalars types you
          are using.
        </Hint>
        <p>
          Mapping for built-in scalar types need not be specified unless you
          want to override them.
        </p>
        <p>Example:</p>
        <Highlight language="yaml">
          {`extensions:
  nitrogql:
    generate:
      type:
        scalarTypes:
          Date: string`}
        </Highlight>
        <p>
          Note that nitrogql supports three different ways to specify the
          mapping:
        </p>
        <Highlight language="yaml">
          {`scalarTypes:
  # 1. Specify as a single string
  Date: string
  # 2. Specify as a pair of send and receive types
  Date:
    send: string | Date
    receive: string
  # 3. Specify as a set of four types
  Date:
    resolverInput: string
    resolverOutput: string | Date
    operationInput: string | Date
    operationOutput: string`}
        </Highlight>
        <p>
          Read more at{" "}
          <Link href="/configuration/scalar-types">
            Configuring Scalar Types
          </Link>
          .
        </p>
        <InPageNav>
          <Link href="/configuration/scalar-types">
            Configuring Scalar Types
          </Link>
        </InPageNav>

        <h4 id="generate.type.allowUndefinedAsOptionalInput">
          allowUndefinedAsOptionalInput
        </h4>
        <p>
          In GraphQL, there is no explicit concept of optional fields. Instead,
          you use fields of nullable types to represent optional fields.
        </p>
        <p>
          If this option is set to <code>true</code>, <code>undefined</code> is
          allowed as an input value for nullable fields. This also implies that
          you can omit optional fields.
        </p>
        <p>
          If this option is set to <code>false</code>, you must explicitly
          provide <code>null</code> for optional fields.
        </p>
        <p>
          This option affects input types (those defined with GraphQL&apos;s{" "}
          <code>input</code> keyword) and operation variables. This option
          defaults to <code>true</code>.
        </p>

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
        fragmentTypeSuffix: ''
        queryVariableSuffix: Query
        mutationVariableSuffix: Mutation
        subscriptionVariableSuffix: Subscription
        fragmentVariableSuffix: ''`}
        </Highlight>

        <h4 id="generate.name.capitalizeOperationNames">
          capitalizeOperationNames
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
          operationResultTypeSuffix
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
          💡 Operation result type is not visible unless you set{" "}
          <code>export.operationResultType</code> to <code>true</code>.
        </Hint>

        <h4 id="generate.name.variablesTypeSuffix">variablesTypeSuffix</h4>
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
          💡 Operation variables type is not visible unless you set{" "}
          <code>export.operationResultType</code> to <code>true</code>.
        </Hint>

        <h4 id="generate.name.fragmentTypeSuffix">fragmentTypeSuffix</h4>
        <p>
          Suffix of the fragment type. Default is <code>&quot;&quot;</code>.
        </p>
        <p>
          For example, if you have <code>fragment PartialUser</code> in your
          schema, the generated fragment type will be{" "}
          <code>PartialUserFragment</code>.
        </p>

        <h4 id="generate.name.queryVariableSuffix">queryVariableSuffix</h4>
        <p>
          Suffix of the query variable. Default is{" "}
          <code>&quot;Query&quot;</code>.
        </p>
        <p>
          For example, if you have <code>query getUser</code> in your schema,
          the generated query variable will be <code>GetUserQuery</code>.
        </p>

        <h4 id="generate.name.mutationVariableSuffix">
          mutationVariableSuffix
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
          subscriptionVariableSuffix
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

        <h4 id="generate.name.fragmentVariableSuffix">
          fragmentVariableSuffix
        </h4>
        <p>
          Suffix of the fragment variable. Default is <code>&quot;&quot;</code>.
        </p>

        <h3 id="generate.export">generate.export</h3>
        <p>
          Set of configurations about how generated code should export generated
          types and variables. Default settings are:
        </p>
        <Highlight language="yaml">
          {`extensions:
  nitrogql:
    generate:
      export:
        defaultExportForOperation: true
        operationResultType: false
        variablesType: false
`}
        </Highlight>

        <h4 id="generate.export.defaultExportForOperation">
          defaultExportForOperation
        </h4>
        <p>
          If <code>true</code>, a generated operation document will be exported
          as a default export. Default is <code>true</code>.
        </p>
        <p>
          For example, if you have <code>query getUser</code> in your schema,
          the generated operation document will be exported as a default export
          so that you can import it like:
        </p>
        <Highlight language="typescript">
          {`// defaultExportForOperation: true
import GetUserQuery from "./app/graphql/queries/getUser.graphql";`}
        </Highlight>
        <p>
          If you set <code>defaultExportForOperations: false</code>, the
          generated operation document will be exported as a named export so
          that you can import it like:
        </p>
        <Highlight language="typescript">
          {`// defaultExportForOperation: false
import { GetUserQuery } from "./app/graphql/queries/getUser.graphql";`}
        </Highlight>

        <h4 id="generate.export.operationResultType">operationResultType</h4>
        <p>
          If <code>true</code>, a generated operation result type will be
          exported. Default is <code>false</code>.
        </p>
        <p>
          For example, if you have <code>query getUser</code> in your schema,
          the generated operation result type will be exported so that you can
          import it like:
        </p>
        <Highlight language="typescript">
          {`// operationResultType: true
import { GetUserResult } from "./app/graphql/queries/getUser.graphql";`}
        </Highlight>
        <Hint>
          💡 You can also use <code>ResultOf</code> from the{" "}
          <code>@graphql-typed-document-node/core</code> package to extract the
          result type from your operation document.
        </Hint>

        <h4 id="generate.export.variablesType">variablesType</h4>
        <p>
          If <code>true</code>, a generated operation variables type will be
          exported. Default is <code>false</code>.
        </p>
        <p>
          For example, if you have <code>query getUser</code> in your schema,
          the generated operation variables type will be exported so that you
          can import it like:
        </p>
        <Highlight language="typescript">
          {`// variablesType: true
import { GetUserVariables } from "./app/graphql/queries/getUser.graphql";`}
        </Highlight>
        <Hint>
          💡 You can also use <code>VariablesOf</code> from the{" "}
          <code>@graphql-typed-document-node/core</code> package to extract the
          variables type from your operation document.
        </Hint>
      </main>
    </Toc>
  );
}
