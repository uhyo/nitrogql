import Link from "next/link";
import { Highlight } from "@/app/_utils/Highlight";
import { Toc } from "../../_toc";
import { Breadcrumb } from "@/app/_utils/Breadcrumb";
import { ogp } from "@/app/_utils/metadata";
import { Hint } from "@/app/_utils/Hint";

export const metadata = ogp({
  title: "nitrogql:model plugin",
});

export default function ModelPlugin() {
  return (
    <Toc>
      <main>
        <Breadcrumb
          parents={[{ label: "References", href: "/references" }]}
          current="nitrogql:model plugin"
        />
        <h2>
          <code>nitrogql:model</code> plugin
        </h2>
        <p>
          <code>nitrogql:model</code> plugin is a built-in plugin for enhancing
          generated resolver types. It helps you to define which fields use{" "}
          <a
            href="https://www.apollographql.com/docs/apollo-server/data/resolvers/#default-resolvers"
            target="_blank"
          >
            default resolvers
          </a>
          . Without this plugin, you have to define all resolvers by yourself,
          which is not practically possible.
        </p>

        <h3 id="usage">Usage</h3>
        <p>
          To use the plugin, you need to add it to the plugins array in the{" "}
          <Link href="/configuration/options">configuration file</Link>.
        </p>
        <Highlight language="yaml">
          {`schema: ./schema/*.graphql
extensions:
  nitrogql:
    plugins:
      - "nitrogql:model"
    # ...`}
        </Highlight>

        <h3 id="model-directive">
          <code>@model</code> directive definition
        </h3>
        <p>
          The plugin adds a <code>@model</code> directive with the following
          definition.
        </p>
        <Highlight language="graphql">
          {`directive @model(
  type: String
) on OBJECT | FIELD_DEFINITION`}
        </Highlight>
        <p>
          This directive is relevant when you are using generated resolver
          types.
        </p>

        <h3 id="using-model-directive-on-fields">
          Using @model directive on fields
        </h3>
        <p>
          An example usage of the <code>@model</code> directive is:
        </p>
        <Highlight language="graphql">
          {`type Query {
  me: User!
}

type User {
  id: ID! @model
  name: String! @model
  email: String!
  posts: [Post!]!
}`}
        </Highlight>
        <p>
          Fields marked with <code>@model</code> directive are considered as{" "}
          <em>part of the model</em>, which means that they are included in data
          interchanged between resolvers.
        </p>
        <p>
          Concretely, resolvers that return a <code>User</code> must return an
          object containing <code>id</code> and <code>name</code> fields. Also,
          resolvers for <code>User</code>&apos;s fields receive an object with
          those fields as the first argument.
        </p>
        <p>
          Also as a consequence, resolvers for model fields need not be defined.
          The default resolver should be able to pick up the appropriate value
          from the model object.
        </p>
        <p>
          With the above setting, the following resolvers implementation is
          considered valid in terms of type safety.
        </p>

        <Highlight language="typescript">
          {`const queryResolvers: Resolvers<Context>["Query"] = {
  me: async () => {
    // returns logged in user
    return {
      id: "1234",
      name: "John Smith",
    };
  }
};

const userResolvers: Resolvers<Context>["User"] = {
  // no need to define resolvers for id and name
  email: async (user) => {
    const userFromDB = await db.findUserById(user.id);
    return userFromDB.email;
  },
  posts: async (user) => {
    const postsFromDB = await db.findPostsByUserId(user.id);
    return postsFromDB;
  }
};`}
        </Highlight>

        <h3 id="using-model-directive-on-object-types">
          Using @model directive on object types
        </h3>
        <p>
          Instead of marking fields with <code>@model</code>, you can apply the{" "}
          <code>@model</code> directive to the whole object type. Doing so will
          replace the model type with one you specify in the <code>type</code>{" "}
          argument. The <code>type</code> argument accepts any string that is a
          valid TypeScript code that represents a type.
        </p>
        <p>
          If you use <code>@model</code> this way, you must define resolvers for
          all fields of the object type.
        </p>
        <p>
          An example usage of the <code>@model</code> directive is:
        </p>
        <Highlight language="graphql">
          {`type Query {
  me: User!
}

type User @model(type: "import('@/models/user').User") {
  id: ID!
  name: String!
  email: String!
  posts: [Post!]!
}`}
        </Highlight>
        <p>
          With the above setting, you would implement resolvers like the
          following.
        </p>
        <Highlight language="typescript">
          {`import { User } from "@/models/user";

const queryResolvers: Resolvers<Context>["Query"] = {
  me: async () => {
    // returns logged in user
    return new User("1234", "John doe");
  }
};

const userResolvers: Resolvers<Context>["User"] = {
  id: (user) => {
    return user.id;
  },
  name: (user) => {
    return user.name;
  },
  email: async (user) => {
    // user is an instance of the User class
    return await user.getEmail();
  },
  posts: async (user) => {
    return await user.getPosts();
  }
};
`}
        </Highlight>

        <h3 id="effects-on-runtime-schema">Effects on Runtime Schema</h3>
        <p>
          The <code>@model</code> directive is purely for type definition
          generation. Therefore, it is removed from the runtime schema (one
          generated by using the{" "}
          <Link href="/configuration/options#generate.serverGraphqlOutput">
            generate.serverGraphqlOutput
          </Link>{" "}
          option).
        </p>

        <Hint>
          🧺 <b>See Also:</b> the{" "}
          <Link href="/blog/release-1.1">blog post for the 1.1 release</Link>{" "}
          also explains the <code>@model</code> directive and its usage in
          detail.
        </Hint>
      </main>
    </Toc>
  );
}
