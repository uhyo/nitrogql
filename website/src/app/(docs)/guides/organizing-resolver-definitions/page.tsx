import { Highlight } from "@/app/_utils/Highlight";
import { Toc } from "../../_toc";
import { Breadcrumb } from "@/app/_utils/Breadcrumb";
import { Hint } from "@/app/_utils/Hint";
import Link from "next/link";
import { ogp } from "@/app/_utils/metadata";

export const metadata = ogp({
  title: "Organizing Resolver Definitions",
});

export default function OrganizingResolverDefinitions() {
  return (
    <Toc>
      <main>
        <Breadcrumb
          parents={[{ label: "Guides", href: "/guides" }]}
          current="Organizing Resolver Definitions"
        />
        <h2>Organizing Resolver Definitions</h2>
        <p>
          This page describes how to organize your resolver definitions into
          separate files. This is useful when you have a large schema with many
          types and fields.
        </p>

        <h3 id="introduction">Introduction</h3>

        <p>
          In order to write server-side code, you can use nitrogql to generate a{" "}
          <Link href="/guides/using-graphql#using-generated-types-from-client-code">
            <code>Resolvers</code>
          </Link>{" "}
          type that contains all the resolvers for your GraphQL schema. When you
          schema is very small, you can write all your resolvers in a single
          file. For example:
        </p>
        <Highlight language="ts">
          {`import { Resolvers } from "@/app/generated/resolvers";

// Context is an object that is passed to all resolvers.
// It is created per request.
type Context = {};

// define all resolvers.
const resolvers: Resolvers<Context> = {
  Query: {
    me: async () => { /* ... */ },
    todos: async () => { /* ... */ }
  },
  Mutation: {
    toggleTodos: async (_, variables) => { /* ... */ }
  },
  User: {
    // ...
  },
};`}
        </Highlight>
        <p>
          As your schema grows, you may want to split your resolvers into
          separate files. While nitrogql does not provide any special support
          for organizing resolvers into separate files, you can use your
          TypeScript wisdom to do this.
        </p>

        <h3 id="defining-resolvers-per-type">Defining resolvers per type</h3>
        <p>
          A straightforward way to split your resolver definitions is to define
          resolvers per type. This can be done with a simple TypeScript syntax:
        </p>
        <Highlight language="ts">
          {`import { Resolvers } from "@/app/generated/resolvers";

type Context = {};

const queryResolvers: Resolvers<Context>["Query"] = {
  me: async () => { /* ... */ },
  todos: async () => { /* ... */ }
};
const mutationResolvers: Resolvers<Context>["Mutation"] = {
  toggleTodos: async (_, variables) => { /* ... */ }
};
const userResolvers: Resolvers<Context>["User"] = {
  // ...
};

// define all resolvers.
const resolvers: Resolvers<Context> = {
  Query: queryResolvers,
  Mutation: mutationResolvers,
  User: userResolvers,
};`}
        </Highlight>
        <p>
          This approach is simple and sometimes sufficient. Splitting resolver
          definitions into smaller ones should improve developer experience
          especially when there are type errors.
        </p>

        <h3 id="splitting-resolvers-into-modules">
          Splitting Resolvers Into Modules
        </h3>
        <p>
          Often times, you may want to organize server-side implementation into
          modules, each of which contains a set of related resolvers, not
          necessarily per type. For example, you may want to have a Todos module
          which will contain the <code>todos</code> resolver in{" "}
          <code>Query</code>, <code>toggleTodos</code> resolver in{" "}
          <code>Mutation</code>, and also fields in <code>Todo</code> type.
        </p>

        <h4 id="exporting-many-resolvers">
          Solution 1: Exporting many resolvers
        </h4>

        <p>
          If you wish to keep it simple, your module can just export each
          resolver as a function:
        </p>
        <Highlight language="ts">
          {`import { Resolvers } from "@/app/generated/resolvers";
import { Context } from "@/app/context";

export const todosResolver: Resolvers<Context>["Query"]["todos"] = async () => {
  // ...
};

export const toggleTodosResolver: Resolvers<Context>["Mutation"]["toggleTodos"] = async (_, variables) => {
  // ...
};

export const todoResolvers: Resolvers<Context>["Todo"] = {
  // ...
};`}
        </Highlight>
        <Hint>
          🔦 <b>Note:</b> Above code assumes that you defined a{" "}
          <code>Context</code> type in <code>@/app/context</code>.
        </Hint>
        <p>
          Then, you can import and use them in your main resolver definition:
        </p>
        <Highlight language="ts">
          {`import { Resolvers } from "@/app/generated/resolvers";
import { Context } from "@/app/context";
import { meResolver } from "./session";
import { todosResolver, toggleTodosResolver, todoResolvers } from "./todos";

// define all resolvers.
const resolvers: Resolvers<Context> = {
  Query: {
    me: meResolver,
    todos: todosResolver,
  },
  Mutation: {
    toggleTodos: toggleTodosResolver,
  },
  Todo: todoResolvers,
  // ...
};`}
        </Highlight>

        <h4 id="defining-resolver-definition-helpers">
          Solution 2: Defining resolver definition helpers
        </h4>

        <p>
          If you rather reduce all those type annotations like{" "}
          <code>
            : Resolvers&lt;Context&gt;[&quot;Query&quot;][&quot;todos&quot;]
          </code>
          , you can define some helper functions to help you define resolvers.
        </p>
        <p>
          Below code defines a <code>partialResolvers</code> function that
          accepts a partial resolver definition and returns it as-is.
        </p>
        <Highlight language="ts">
          {`import { Resolvers } from "@/app/generated/resolvers";
import { Context } from "@/app/context";

function partialResolvers<
  R extends Partial<{
    [K in keyof Resolvers<Context>]: Partial<Resolvers<Context>[K]>;
  }>
>(resolvers: R): R {
  return resolvers;
}`}
        </Highlight>
        <p>Then, you can use it in your module:</p>
        <Highlight language="ts">
          {`
export const todosResolvers = partialResolvers({
  Query: {
    todos: async () => {
      // ...
    }
  },
  Mutation: {
    toggleTodos: async (_, variables) => {
      // ...
    }
  },
  Todo: {
    // ...
  }
});`}
        </Highlight>
        <p>
          This way, you still receive the benefit of type checking and
          contextual typing, while being able to define only resolvers that you
          want to define in the module. Thanks to the definition of{" "}
          <code>partialResolvers</code>, the resulting type of{" "}
          <code>todosResolvers</code> keeps track of which resolvers are
          defined.
        </p>
        <p>
          Then, you need to gather all resolvers from modules and merge them
          together. To do this in a type-safe way, you can define another helper
          function:
        </p>
        <Highlight language="ts">
          {`type UnionToIntersection<U> = (
  U extends unknown ? (k: U) => void : never
) extends (k: infer I) => void
  ? I
  : never;

function mergeResolvers<
  const Rs extends readonly Partial<{
    [K in keyof Resolvers<Context>]: Partial<Resolvers<Context>[K]>;
  }>[]
>(
  resolvers: Rs
): {
  [K in keyof Resolvers<Context>]: UnionToIntersection<
    Rs[number] extends infer U
      ? U extends Record<K, infer V>
        ? V
        : never
      : never
  > & {};
} {
  const result = {} as Record<string, Record<string, unknown>>;
  for (const resolver of resolvers) {
    for (const [key, value] of Object.entries(resolver)) {
      if (result[key] === undefined) {
        result[key] = value;
      } else {
        Object.assign(result[key], value);
      }
    }
  }
  return result as any;
}`}
        </Highlight>
        <p>Then, you can use it in your main resolver definition:</p>
        <Highlight language="ts">
          {`import { Resolvers } from "@/app/generated/resolvers";
import { Context } from "@/app/context";
import { sessionResolvers } from "./session";
import { todosResolvers } from "./todos";

// Note: \`: Resolvers<Context>\` is important to guarantee that you
// defined all required resolvers.
const resolvers: Resolvers<Context> = mergeResolvers([
  sessionResolvers,
  todosResolvers,
]);`}
        </Highlight>
        <p>
          This way, you can split resolver definitions as you like, while
          maintaining type safety. Only caveat is that these helper functions
          look too magical. 😅
        </p>

        <Hint>
          👻 <b>Note:</b> above helpers are not thoroughly tested. You can use
          them in your project, but please be aware that they might have some
          bugs.
        </Hint>
        <Hint>
          ⚖️ <b>Note:</b> Code in this page is licensed under the MIT license.
        </Hint>
      </main>
    </Toc>
  );
}
