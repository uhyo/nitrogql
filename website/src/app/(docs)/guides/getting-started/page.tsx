import Image from "next/image";
import Link from "next/link";
import { Hint } from "@/app/_utils/Hint";
import { Highlight } from "@/app/_utils/Highlight";
import { Figures } from "@/app/_utils/Figures";
import ScreenshotGeneratedTypes from "./figures/screenshot-generated-types.png";
import { Toc } from "../../_toc";
import { Breadcrumb } from "@/app/_utils/Breadcrumb";

export default function GettingStarted() {
  return (
    <Toc>
      <main>
        <Breadcrumb
          parents={[{ label: "Guides", href: "/guides" }]}
          current="Getting Started"
        />
        <h2>Getting Started</h2>
        <p>
          This page guides you through the steps to get started with nitrogql.
        </p>

        <Hint>
          🧑‍🏫 Currently, this guide assumes that you are already familiar with
          GraphQL.
        </Hint>

        <h3 id="installation">Installation</h3>
        <p>
          The nitrogql CLI can be installed with npm. The CLI is required for
          any workflow using nitrogql. Since generated types depend on{" "}
          <a
            href="https://github.com/dotansimha/graphql-typed-document-node"
            target="_blank"
          >
            TypedDocumentNode
          </a>
          , you also need to install it to your project.
        </p>
        <Highlight language="bash">{`npm install --save-dev @nitrogql/cli @graphql-typed-document-node/core`}</Highlight>
        <p>
          Depending on framework or tool you use, you may also need to install
          appropriate loader packages.
        </p>
        <p>
          For <b>webpack-based tools</b>:
        </p>
        <Highlight language="bash">{`npm install --save-dev @nitrogql/graphql-loader`}</Highlight>
        <p>
          For <b>Rollup-based tools</b>:
        </p>
        <Highlight language="bash">{`npm install --save-dev @nitrogql/rollup-plugin`}</Highlight>

        <h3 id="configuration-file">Configuration File</h3>
        <p>
          nitrogql uses a configuration file to specify the location of your
          schema and operations. The configuration file is named{" "}
          <code>graphql.config.yaml</code>. The configuration file should be
          placed in the root of your project.
        </p>
        <p>The simplest configuration file will look like:</p>
        <Highlight language="yaml">
          {`schema: ./schema/*.graphql
documents: ./app/**/*.graphql`}
        </Highlight>
        <p>
          This file follows{" "}
          <a href="https://the-guild.dev/graphql/config/docs" target="_blank">
            the GraphQL Config convention
          </a>{" "}
          from The Guild. You might already have a configuration file if you use
          other GraphQL tools.
        </p>
        <p>
          See <Link href="/configuration">Configuration</Link> for full details
          about the configuration file. You can also use JSON (
          <code>graphql.config.json</code>) or JavaScript (
          <code>graphql.config.js</code>) instead of YAML.
        </p>

        <h3 id="setting-up-graphql-loader-for-webpack">
          Setting up GraphQL loader for webpack
        </h3>
        <p>
          If you are using webpack-based tools, you need to configure the
          loader.
        </p>
        <p>Example setup (webpack.config.js):</p>
        <Highlight language="javascript">
          {`module: {
  rules: [
    // ...
    {
      test: /\\.graphql$/,
      loader: "@nitrogql/graphql-loader",
      options: {
        configFile: "./graphql.config.yaml",
      }
    }
  ]
}`}
        </Highlight>
        <Hint>
          🔥 We have a Next.js example:{" "}
          <a
            href="https://github.com/uhyo/nitrogql/tree/master/examples/nextjs"
            target="_blank"
          >
            see on GitHub
          </a>
        </Hint>

        <h3 id="setting-up-graphql-loader-for-rollup">
          Setting up GraphQL loader for Rollup
        </h3>
        <p>
          If you are using Rollup-based tools, you need to configure the plugin.
        </p>
        <p>Example setup (rollup.config.js):</p>
        <Highlight language="javascript">
          {`import nitrogql from "@nitrogql/rollup-plugin";

export default {
  plugins: [
    nitrogql({
      include: ["**/*.graphql"],
    }),
  ],
};
`}
        </Highlight>
        <Hint>
          🔥 We have a Vite example:{" "}
          <a
            href="https://github.com/uhyo/nitrogql/tree/master/examples/vite"
            target="_blank"
          >
            see on GitHub
          </a>
        </Hint>

        <h3 id="writing-your-schema">Writing your schema</h3>
        <p>
          nitrogql is a tool for the <strong>schema-first approach</strong> to
          GraphQL. This means that you write your schema first to define your
          GraphQL API.
        </p>
        <p>
          Place your schema files in the directory specified by the{" "}
          <code>schema</code> field in the configuration file.
        </p>
        <Highlight language="graphql">
          {`# ./schema/todo.graphql
"One todo item."
type Todo {
  "ID of this todo item."
  id: ID!
  "Contents of this todo item."
  body: String!
  "When not null, date when this item was marked done."
  finishedAt: Date
  "Date when this item was created."
  createdAt: Date!
}`}
        </Highlight>
        <Hint>
          💡 Check out{" "}
          <a
            href="https://github.com/uhyo/nitrogql/tree/master/examples/nextjs"
            target="_blank"
          >
            our examples
          </a>{" "}
          if you just want to try out nitrogql.
        </Hint>

        <h3 id="writing-operations">Writing operations</h3>
        <p>
          Once you have your schema, you can write GraphQL operations. Place
          your operation files in the directory specified by the{" "}
          <code>documents</code> field in the configuration file.
        </p>
        <p>
          nitrogql recommends having separate operation files (
          <code>.graphql</code>) instead of embedding them in your code.
        </p>
        <Highlight language="graphql">
          {`# ./app/getTodos.graphql
query getTodos {
  todos {
    id
    body
    createdAt
    finishedAt
  }
}`}
        </Highlight>

        <h3 id="statically-checking-graphql-files">
          Statically checking GraphQL files
        </h3>
        <p>
          To check GraphQL files, run the following command in your project
          directory:
        </p>
        <Highlight language="bash">{`npx nitrogql check`}</Highlight>
        <Hint>
          💡 <b>Note</b>: nitrogql CLI looks for the configuration file in the
          current directory by default. To specify the location of the
          configuration file, use the <code>--config-file</code> option.
        </Hint>
        <p>
          If all GraphQL files are valid, the command will exit with code 0.
          Otherwise, it will print errors and exit with code 1.
        </p>

        <h3 id="generating-typescript-types">Generating TypeScript types</h3>
        <p>
          nitrogql generates one <code>.d.ts</code> file for the whole schema,
          and one <code>.d.graphql.ts</code> for each operation file.{" "}
          <code>.d.graphql.ts</code> files are placed next to operation files.
        </p>
        <Hint>
          💡 <b>Note</b>: <code>.d.graphql.ts</code> files are supported by
          TypeScript 5.0 or later. If you are using TypeScript 4.x, you need to
          configure nitrogql to generate <code>.d.ts</code> files instead. See{" "}
          <Link href="/configuration">Configuration</Link> for details.
        </Hint>
        <p>
          Before generating types, you need to specify the location of generated
          schema file in the configuration file:
        </p>
        <Highlight language="yaml">
          {`schema: ./schema/*.graphql
documents:
extensions:
  nitrogql:
    generate:
      # Specify the location of generated schema file.
      schemaOutput: ./src/generated/schema.d.ts`}
        </Highlight>
        <p>
          There is no option to specify the location of generated types for
          operation files; they are always placed next to operation files.
        </p>
        <p>
          To generate TypeScript types, run the following command in your
          project directory:
        </p>
        <Highlight language="bash">{`npx nitrogql generate`}</Highlight>
        <Hint>
          💡 <b>Note</b>: the <code>generate</code> command implies{" "}
          <code>check</code>. If there are errors in GraphQL files, the command
          fails and does not generate any files.
        </Hint>
        <p>
          After a successful run, you will see types (<code>.d.graphql.ts</code>
          ) and source maps (<code>.d.graphql.ts.map</code>) generated next to
          your operation files.
        </p>
        <Figures>
          <figure>
            <Image
              src={ScreenshotGeneratedTypes}
              width="640"
              alt="Screenshot of VSCode showing generated types"
            />
            <figcaption>
              Type definitions and source maps are generated next to each
              operation.
            </figcaption>
          </figure>
        </Figures>

        <h3 id="using-generated-types">Using generated types</h3>
        <p>
          Now you can import <code>.graphql</code> files in your TypeScript
          code, but probably you need to adjust your <code>tsconfig.json</code>{" "}
          so that TypeScript allows importing <code>.graphql</code> files.
        </p>
        <Highlight language="diff">
          {` {
   "compilerOptions": {
     // ...
+    "allowArbitraryExtensions": true,
     // ...
   }
 }`}
        </Highlight>
        <p>
          After configuring TypeScript correctly, it&apos;s time to import{" "}
          <code>.graphql</code> files in your code. With default settings, these
          files export <code>TypedDocumentNode</code> objects as default
          exports. You can use them with any GraphQL client library. Below are
          examples for Apollo Client and React:
        </p>
        <Highlight language="typescript">
          {`import { useQuery } from "@apollo/client";
import getTodosQuery from "./getTodos.graphql";

export function SampleComponent() {
  const { data } = useQuery(getTodosQuery);
  
  return <ul>{
    data?.todos.map(
      (todo) => <li key={todo.id}>{todo.body}</li>
    )
  }</ul>;
}`}
        </Highlight>
        <p>
          Of course this is type-safe. If your code accesses a field that does
          not exist in the schema, TypeScript will report an error.
        </p>

        <h3 id="watching-and-generated-types-automatically">
          Watching and generating types automatically
        </h3>
        <p>
          It is tedious to run <code>generate</code> command every time you
          change GraphQL files. Unfortunately, nitrogql does not provide a
          built-in way to watch GraphQL files and generate types automatically.
          However, you can use{" "}
          <a
            href="https://github.com/open-cli-tools/chokidar-cli"
            target="_blank"
          >
            chokidar-cli
          </a>{" "}
          or similar tools to watch GraphQL files and run <code>generate</code>{" "}
          command automatically:
        </p>
        <Highlight language="bash">
          {`chokidar '**/*.graphql' --initial --command 'npx nitrogql generate'`}
        </Highlight>
        <p>
          Alternatively, you can use{" "}
          <a href="https://marketplace.visualstudio.com/items?itemName=emeraldwalk.RunOnSave">
            Run on Save
          </a>{" "}
          VSCode extension to run <code>generate</code> command automatically
          when you save a GraphQL file. Example configuration:
        </p>
        <Highlight language="json">
          {`{
  "emeraldwalk.runonsave": {
    "commands": [
      {
        "match": "\\\\.graphql$",
        "cmd": "npx nitrogql generate"
      }
    ]
  }
}`}
        </Highlight>

        <Hint>
          🧺 <b>Read Next</b>: <Link href="/configuration">Configuration</Link>,{" "}
          <Link href="/cli">CLI Usage</Link>
        </Hint>
      </main>
    </Toc>
  );
}
