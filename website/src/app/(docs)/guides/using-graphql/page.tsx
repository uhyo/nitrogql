import Image from "next/image";
import Link from "next/link";
import { Hint } from "@/app/_utils/Hint";
import { Highlight } from "@/app/_utils/Highlight";
import { Figures } from "@/app/_utils/Figures";
import ScreenshotGeneratedTypes from "./figures/screenshot-generated-types.png";
import { Toc } from "../../_toc";
import { Breadcrumb } from "@/app/_utils/Breadcrumb";
import { ogp } from "@/app/_utils/metadata";

export const metadata = ogp({
  title: "Using GraphQL in TypeScript projects",
});

export default function UsingGraphQL() {
  return (
    <Toc>
      <main>
        <Breadcrumb
          parents={[{ label: "Guides", href: "/guides" }]}
          current="Using GraphQL in TypeScript projects"
        />
        <h2>Using GraphQL in TypeScript projects</h2>
        <p>
          After{" "}
          <Link href="/guides/getting-started">
            installing nitrogql to your TypeScript project
          </Link>
          , you can start using GraphQL in your project. This page guides you
          how to use GraphQL with nitrogql.
        </p>

        <Hint>
          🧑‍🏫 Currently, this guide assumes that you are already familiar with
          GraphQL.
        </Hint>
        <Hint>
          💡 Check out{" "}
          <a
            href="https://github.com/uhyo/nitrogql/tree/master/examples/nextjs"
            target="_blank"
          >
            our examples
          </a>{" "}
          for a working example.
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

        <h3 id="writing-operations">Writing operations</h3>
        <p>
          Once you have your schema, you can write GraphQL operations. Place
          your operation files in the directory specified by the{" "}
          <code>documents</code> field in the configuration file.
        </p>
        <p>
          nitrogql recommends having separate operation files (
          <code>.graphql</code>) instead of embedding them in your code. This is
          the only way of writing GraphQL operations in nitrogql.
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

        <h3 id="generating-typescript-files">Generating TypeScript files</h3>
        <p>nitrogql generates several TypeScript files from GraphQL files:</p>
        <p>
          <b>Schema type definition file</b> is configured by the{" "}
          <Link href="/configuration/options#generate.schemaOutput">
            <code>generate.schemaOutput</code>
          </Link>{" "}
          option. It is a single <code>.d.ts</code> file that contains type
          definitions for all types in the schema, so it must be generated.
        </p>
        <p>
          <b>Resolver type definition file</b> is configured by the{" "}
          <Link href="/configuration/options#generate.resolversOutput">
            <code>generate.resolversOutput</code>
          </Link>{" "}
          option. It is a single <code>.d.ts</code> file that contains type
          definitions for all resolvers. It is optional, and only generated when
          you specify the option. Resolver types are useful when you are
          developing a GraphQL server.
        </p>
        <p>
          <b>Server GraphQL schema file</b> is configured by the{" "}
          <Link href="/configuration/options#generate.serverGraphqlOutput">
            <code>generate.serverGraphqlOutput</code>
          </Link>{" "}
          option. It is a <code>.ts</code> file that exports the entire GraphQL
          schema as a single string (in SDL format). It can be used at runtime
          to initialize a GraphQL server without having to manually load and
          concatenate all schema files. It is optional, and only generated when
          you specify the option.
        </p>
        <p>
          <b>Operation type definition files</b> are generated for each
          operation file. They are always emitted as long as you configure
          nitrogql to load operation files (via the{" "}
          <Link href="/configuration/options#schema-operations">
            <code>documents</code>
          </Link>{" "}
          option). By default, operation type definition files are{" "}
          <code>.d.graphql.ts</code> files and are placed next to each operation
          file.
        </p>
        <Hint>
          💡 <b>Note</b>: <code>.d.graphql.ts</code> files are supported by
          TypeScript 5.0 or later. If you are using TypeScript 4.x, you need to
          configure nitrogql to generate <code>.d.ts</code> files instead. See{" "}
          <Link href="/configuration">Configuration</Link> for details.
        </Hint>
        <p>
          To generate these files, you need to specify the location of generated
          files in the configuration file:
        </p>
        <Highlight language="yaml">
          {`schema: ./schema/*.graphql
documents:
extensions:
  nitrogql:
    generate:
      # Specify the location of generated schema file.
      schemaOutput: ./src/generated/schema.d.ts
      # Specify the location of generated resolver file.
      resolversOutput: ./src/generated/resolvers.d.ts
      # Specify the location of generated server GraphQL schema file.
      serverGraphqlOutput: ./src/generated/graphql.ts
      `}
        </Highlight>
        <p>
          There is no option to specify the location of generated types for
          operation files; they are always placed next to operation files.
        </p>
        <p>Then, run the following command in your project directory:</p>
        <Highlight language="bash">{`npx nitrogql generate`}</Highlight>
        <Hint>
          💡 <b>Note</b>: the <code>generate</code> command implies{" "}
          <code>check</code>. If there are errors in GraphQL files, the command
          fails and does not generate any files.
        </Hint>
        <p>
          After a successful run, you will see generated files in the specified
          locations.
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

        <h3 id="using-generated-types-from-client-code">
          Using generated types from client code
        </h3>
        <p>
          With the help of the generated operation type definition files,
          nitrogql allows you to use GraphQL operations type-safely in your
          TypeScript code. To use them, you directly import{" "}
          <code>.graphql</code> files in your code.
        </p>
        <p>
          However, probably you need to adjust your <code>tsconfig.json</code>{" "}
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
          <code>.graphql</code> files. With default settings, these files export{" "}
          a <code>TypedDocumentNode</code> object as a default export. You can
          use it with any GraphQL client library. Below is an example with
          Apollo Client and React:
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
          In this example <code>getTodos.graphql</code> is an operation file
          that contains a GraphQL query (<code>query getTodos {"{ ... }"}</code>
          ). By passing the exported query object to <code>useQuery</code>, you
          can execute the query.
        </p>
        <p>
          Of course this is type-safe. The type of the query result,{" "}
          <code>data</code>, precisely matches the shape of the query. This
          means that if your code tries to access a field that does not exist in
          the schema, or is not fetched by the operation, TypeScript will report
          an error.
        </p>

        <h3 id="using-generated-types-from-server-code">
          Using generated types from server code
        </h3>
        <p>
          In the schema-first approach to GraphQL, you develop a GraphQL server
          so that it implements the schema you wrote. Typically this is done by
          writing <strong>resolvers</strong>. The generated resolver type
          definition file helps you write resolvers in a type-safe manner.
        </p>
        <Hint>
          🍰 Below guide uses{" "}
          <a
            href="https://www.apollographql.com/docs/apollo-server/"
            target="_blank"
          >
            Apollo Server
          </a>{" "}
          as an example, but you can use any GraphQL server library.
        </Hint>
        <p>
          With nitrogql, the basic setup of a GraphQL server will look like:
        </p>
        <Highlight language="ts">
          {`import { ApolloServer } from "@apollo/server";
// server GraphQL schema file
import { schema } from "@/app/generated/graphql";
// resolver types
import { Resolvers } from "@/app/generated/resolvers";

// Context is an object that is passed to all resolvers.
// It is created per request.
type Context = {};

// define all resolvers.
const resolvers: Resolvers<Context> = {
  Query: {
    todos: async () => { /* ... */ }
  },
  Mutation: {
    toggleTodos: async (_, variables) => { /* ... */ }
  },
  // ...
};

const server = new ApolloServer({
  typeDefs: schema,
  resolvers,
});
// ...
`}
        </Highlight>
        <p>
          Of course, you can use any TypeScript technique you know to
          organize/structure your code. For example, you might want to define
          Query resolvers and Mutation resolvers separately:
        </p>
        <Highlight language="ts">
          {`const queryResolvers: Resolvers<Context>["Query"] = {
  // ...
};
const mutationResolvers: Resolvers<Context>["Mutation"] = {
  // ...
};
const resolvers = {
  Query: queryResolvers,
  Mutation: mutationResolvers,
  // ...
};`}
        </Highlight>
        <p>
          <strong>However</strong>, at this step the generated resolver type
          definition is not practically usable because it does not allow use of{" "}
          <a
            href="https://www.apollographql.com/docs/apollo-server/data/resolvers/#default-resolvers"
            target="_blank"
          >
            default resolvers
          </a>
          . This means that you need to define resolvers for every single field
          of every object type in the schema. This isn&apos;t what you usually
          do.
        </p>
        <p>
          To mitigate this problem, nitrogql provides the{" "}
          <code>nitrogql:model</code> plugin. This plugin allows you to use a{" "}
          <code>@model</code> directive in your schema to mark a field as
          included in the model. Fields with this directive are to be resolved
          by default resolvers, so you don&apos;t need to define resolvers for
          them.
        </p>
        <p>
          This may not be something familiar to you, but it is needed for making
          it practical to write resolvers while maintaining the perfect type
          safety.
        </p>
        <p>
          For details about the <code>nitrogql:model</code> plugin, see{" "}
          <Link href="/references/plugin-model">
            <code>nitrogql:model</code> plugin
          </Link>
          .
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
