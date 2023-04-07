import Image from "next/image";
import Link from "next/link";
import { Hint } from "@/app/(utils)/Hint";
import { Highlight } from "@/app/(utils)/Highlight";
import { Figures } from "@/app/(utils)/Figures";

export default function GettingStarted() {
  return (
    <main>
      <h2>Configuration</h2>
      <p>
        nitrogql uses a configuration file to specify the location of your
        schema and operations, and to configure how types are generated. The
        configuration file is named <code>graphql.config.ts</code>. The
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

      <h3>schema and operations</h3>
      <p>
        To specify the location of your schema and operations, use{" "}
        <code>schema</code> and <code>documents</code> top-level fields. These
        fields accept glob patterns and can be specified as a string or an array
        of strings.
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
        💡 Other configuration options are placed under
        <code>extensions.nitrogql</code> in the configuration file.
      </Hint>

      <h3>generate.schemaOutput</h3>
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

      <h3>generate.mode</h3>
      <p>
        Configures how types for operations are generated. Possible values are:
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
        <code>foo.graphql</code> which allows importing <code>foo.graphql</code>{" "}
        as a module.
      </p>
      <Hint>
        💡 With this mode, you need to configure <code>tsconfig.json</code> to
        enable the <code>allowArbitraryExtensions</code> compiler option.
      </Hint>
      <p>
        In order to import <code>.graphql</code> files as modules, you also need
        to configure your bundler to handle <code>.graphql</code> files. See{" "}
        <Link href="/getting-started">Getting Started</Link>.
      </p>

      <h4>with-loader-ts-4.0</h4>
      <p>Generates type definitions compatible with TypeScript 4.x.</p>
      <p>
        This mode generates <code>foo.graphql.d.ts</code> next to{" "}
        <code>foo.graphql</code> which allows importing <code>foo.graphql</code>
        as a module.
      </p>
      <p>
        In order to import <code>.graphql</code> files as modules, you also need
        to configure your bundler to handle <code>.graphql</code> files. See{" "}
        <Link href="/getting-started">Getting Started</Link>.
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

      <h3>generate.scalarTypes</h3>
      <p>
        Configures how GraphQL scalar types are mapped to TypeScript types. The
        default mapping is:
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
        If you declare a custom scalar type in your schema, you must specify the
        mapping in the configuration file. Any TypeScript code is allowed as
        long as it is valid as a type.
      </p>
      <p>
        Mapping for built-in scalar types need not be specified unless you want
        to override them.
      </p>
      <p>Example:</p>
      <Highlight language="yaml">
        {`extensions:
  nitrogql:
    generate:
      scalarTypes:
        Date: Date`}
      </Highlight>
    </main>
  );
}
