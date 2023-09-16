import { Highlight } from "@/app/_utils/Highlight";
import { ArticleMetadata } from "./_meta";
import Link from "next/link";
import { Hint } from "@/app/_utils/Hint";

export const blogPostRelease1_1: ArticleMetadata = {
  slug: "release-1.1",
  title: "nitrogql 1.1 release: hello type-safe resolvers!",
  shortDescription: `In nitrogql 1.1, we added support for generating type definitions for resolvers.
This means that you can now get type safety on both the client-side and the server-side.`,
  publishDate: new Date("2023-09-16T00:00Z"),
  render,
};

function render() {
  return (
    <>
      <p>
        Today, we are happy to announce release of <strong>nitrogql 1.1</strong>
        !
      </p>
      <p>
        <b>nitrogql</b> is a toolchain for using GraphQL in TypeScript projects.
        In 1.1, we added support for generating type definitions for resolvers.
        This means that you can now get type safety on both the client-side and
        the server-side using nitrogql.
      </p>

      <h3 id="new-in-nitrogql-1-1">New in nitrogql 1.1</h3>
      <p>
        nitrogql 1.1 can generate two more TypeScript files than 1.0. They are:
      </p>
      <ul>
        <li>
          <b>Resolvers type definition file</b> defines what resolvers you
          should implement.
        </li>
        <li>
          <b>Server GraphQL schema file</b> makes it easy to supply your GraphQL
          schema to a GraphQL server at runtime.
        </li>
      </ul>
      <p>
        These files are helpful for implementing GraphQL servers. In
        nitrogql&apos;s
        <strong>schema-first</strong> approach, you write your GraphQL schema
        first, and then implement both the client and the server from it. The
        release of 1.1 fills the gap on the server-side; you can now get type
        safety for both the client and the server!
      </p>

      <h3 id="configuring-nitrogql-for-server-side-development">
        Configuring nitrogql for server side development
      </h3>
      <p>
        To generate the new files, you need to add some options to your{" "}
        <Link href="/configuration/options">configuration file</Link>. Namely,
        add <code>resolversOutput</code> and <code>serverGraphqlOutput</code>{" "}
        under the <code>generate</code> option.
      </p>
      <Highlight language="yaml">
        {`schema: ./schema/*.graphql
documents: ./src/**/*.graphql
extensions:
  nitrogql:
    plugins:
      - "nitrogql:model"
    generate:
      schemaOutput: ./src/generated/schema.d.ts
      resolversOutput: ./src/generated/resolvers.d.ts
      serverGraphqlOutput: ./src/generated/server-graphql.ts
      # ...`}
      </Highlight>
      <p>
        With this setting, <code>nitrogql generate</code> will generate{" "}
        <code>resolvers.d.ts</code> and <code>server-graphql.ts</code>.
      </p>

      <h3 id="writing-resolvers-with-type-safety">
        Writing resolvers with type safety
      </h3>
      <p>
        The generated <code>resolvers.d.ts</code> helps you write resolvers with
        type safety. It exports a <code>Resolvers</code> type, which is the type
        of the resolvers object you should implement. Suppose you have a schema
        like:
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
}

type Post {
  id: ID! @model
  title: String! @model
  content: String!
}`}
      </Highlight>
      <p>
        Then, you can use the <code>Resolvers</code> type:
      </p>
      <Highlight language="typescript">
        {`import { Resolvers } from "./generated/resolvers";

type Context = {};

const resolvers: Resolvers<Context> = {
  Query: {
    me: async () => {
      // Returns the current user.
      return {
        id: "12345",
        name: "uhyo",
      }
    },
  },
  User: {
    email: async (user) => {
      const dbUser = await db.getUser(user.id);
      return dbUser.email;
    },
    posts: async (user) => {
      const dbPosts = await db.getPostsByUser(user.id);
      return dbPosts;
    },
  },
  Post: {
    content: async (post) => {
      const dbPost = await db.getPost(post.id);
      return dbPost.content;
    }
  },
};`}
      </Highlight>
      <p>
        Note that the <code>Resolvers</code> type is a generic type that takes
        the type of the context object as the type argument. Context is an
        object that is created per request and passed to all resolvers. You can
        use it to pass session information, database connections, and so on to
        resolvers.
      </p>

      <h3 id="model-directive">
        Wait, what the hell is that <code>@model</code> thing?
      </h3>
      <p>
        Yeah you noticed that something unfamiliar is in the schema. It&apos;s
        the <code>@model</code> directive. It&apos;s a directive added by
        nitrogql (more specifically, the{" "}
        <Link href="/references/plugin-model">
          <code>nitrogql:model</code> plugin
        </Link>
        ). This directive has been introduced along with the release of 1.1.
      </p>
      <p>
        The fields marked with the <code>@model</code> directive are part of{" "}
        <i>the model object</i> of that type. This has two implications:
      </p>
      <ul>
        <li>
          You don&apos;t need to implement resolvers for the fields marked with{" "}
          <code>@model</code> directive.{" "}
          <a
            href="https://www.apollographql.com/docs/apollo-server/data/resolvers/#default-resolvers"
            target="_blank"
          >
            Default resolvers
          </a>{" "}
          will take care of them.
        </li>
        <li>
          When you return an object of that type from a resolver, you need to
          include all the fields marked with <code>@model</code> directive.
        </li>
      </ul>
      <p>
        The <code>@model</code> directive exists for keeping it both practical
        and type-safe to implement resolvers. For type safety, we must ensure
        that resolvers are implemented for <em>all</em> the fields in the
        schema; otherwise it would be a runtime error. However, it&apos;s not
        practical to implement resolvers for every single field. That would be a
        lot of boilerplate code like <code>id: (user) =&gt; user.id</code>. This
        is where the <b>default resolver</b> comes in. The default resolver will
        behave as if you implemented that trivial resolver.
      </p>
      <p>
        The <code>@model</code> directive is a way to tell nitrogql that you
        would like to use the default resolver for that field. nitrogql will
        recognize that directive and remove the field from the list of resolvers
        you need to implement. The point is that it is up to <em>you</em> to
        decide which resolver to implement and which resolver to leave for the
        default resolver. That&apos;s why <em>you</em> need to mark the fields
        manually with the <code>@model</code> directive. nitrogql didn&apos;t
        choose to implement some heuristic to automatically decide which fields
        use the default resolver. It would not be flexible enough.
      </p>
      <p>
        As a consequence of the use of default resolvers, you need to include
        all the fields marked with <code>@model</code> directive to any object
        you return from a resolver (this is what we call <i>the model object</i>
        ). This is because the default resolver will not be able to resolve the
        fields that you did not include in the object.
      </p>
      <p>
        You know, GraphQL resolvers form a chain during the execution of a
        GraphQL query. When you return an object from a resolver, the next
        resolver in the chain will receive that object as the parent object.
        That&apos;s why resolvers receive the model object as the first
        argument. The <code>@model</code> directive affects communication
        between resolvers in this way.
      </p>

      <h4 id="usage-of-model">
        Usage of <code>@model</code>
      </h4>
      <p>
        Now you know what the <code>@model</code> directive is for. Let&apos;s
        review the example above again. 😉
      </p>
      <p>
        Looking at the schema, the model object for <code>User</code> type
        includes <code>id</code> and <code>name</code> fields. The{" "}
        <code>email</code> and <code>posts</code> fields are not included in the
        model object. Likewise, the model object for <code>Post</code> type
        includes <code>id</code> and <code>title</code> fields, but not{" "}
        <code>content</code>.
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
}

type Post {
  id: ID! @model
  title: String! @model
  content: String!
}`}
      </Highlight>
      <p>
        Next, look at the implementation of the <code>me</code> resolver. It
        returns an object that includes <code>id</code> and <code>name</code>{" "}
        fields. That&apos;s fine because the model object for <code>User</code>{" "}
        type includes those fields.
      </p>
      <Highlight language="typescript">
        {`  Query: {
    me: async () => {
      // Returns the model object for User
      return {
        id: "12345",
        name: "uhyo",
      }
    },
  },`}
      </Highlight>
      <p>
        Looking at <code>User</code> resolvers, the <code>email</code> and{" "}
        <code>posts</code> resolvers are implemented because they are not marked
        with <code>@model</code> directive.
      </p>
      <Highlight language="typescript">
        {` User: {
    email: async (user) => {
      // user is of type { id: string; name: string }
      const dbUser = await db.getUser(user.id);
      return dbUser.email;
    },
    posts: async (user) => {
      const dbPosts = await db.getPostsByUser(user.id);
      return dbPosts;
    },
  },`}
      </Highlight>
      <p>
        As mentioned above, the <code>user</code> argument is the model object
        for <code>User</code> type. Thus it includes <code>id</code> and{" "}
        <code>name</code> fields. The <code>email</code> resolver uses the{" "}
        <code>id</code> field to fetch the email address from the database.
      </p>
      <p>
        You can think of non-model field resolvers as another round of data
        fetching. The <code>id</code> field is the key for fetching more data
        from the database. <code>User</code>-returning resolvers include{" "}
        <code>id</code> field in the model object, so later round resolvers
        (like <code>email</code> and <code>posts</code>) can use it to fetch
        more data. In a practical situation you might use techniques like{" "}
        <a href="https://github.com/graphql/dataloader" target="_blank">
          DataLoader
        </a>{" "}
        to optimize the data fetching, but the same principle applies.
      </p>
      <p>
        In this sense, the <code>id</code> field in <code>User</code> is{" "}
        <em>necessarily</em> included in the model object. On the other hand,
        the <code>name</code> field, which is not used for fetching more data,
        is not quite necessary to be included in the model object.
      </p>
      <p>
        Then, why is <code>name</code> included in the model object? You know,
        this is for the sake of optimization. If <code>name</code> is queried so
        often, it would be better to fetch it in the <i>first round</i> of data
        fetching (i.e. the <code>me</code> resolver). If it&apos;s not included
        in the model object, another round of data fetching would be required to
        fetch the name. By utilizing the <code>@model</code> directive, you can
        easily optimize the data fetching still maintaining the type safety.
        Nicer optimization would require examining the whole query before
        entering the chain of resolvers, but that won&apos;t be that easy.
      </p>

      <h4 id="replacing-whole-model-object-type-with-model">
        Replacing whole model object type with <code>@model</code>
      </h4>
      <p>
        If you are diligent enough, you might have been defining dedicated model
        classes for each type. For example, you might have been writing code
        like:
      </p>
      <Highlight language="typescript">
        {`class User {
  readonly id: string;
  readonly name: string;

  constructor(id: string, name: string) {
    this.id = id;
    this.name = name;
  }
  async getEmail() {
    const dbUser = await db.getUser(this.id);
    return dbUser.email;
  }
  async getPosts() {
    const dbPosts = await db.getPostsByUser(this.id);
    return dbPosts;
  }
}`}
      </Highlight>
      <p>
        nitrogql supports this mode of defining models, though not very
        recommended. This is similar to GraphQL Code Generator&apos;s{" "}
        <a
          href="https://the-guild.dev/graphql/codegen/plugins/typescript/typescript-resolvers#use-your-model-types-mappers"
          target="_blank"
        >
          <code>mappers</code> option
        </a>
        .
      </p>
      <p>
        To use this class as the model object, you can use the{" "}
        <code>@model</code> directive on the entire type. For example:
      </p>
      <Highlight language="graphql">
        {`type User @model(type: "import('@/model/user').User") {
  id: ID!
  name: String!
  email: String!
  posts: [Post!]!
}`}
      </Highlight>
      <p>
        This tells nitrogql that the model object for GraphQL <code>User</code>{" "}
        type is an instance of the <code>User</code> class. With this setting,
        your resolver implementation can be like:
      </p>
      <Highlight language="typescript">
        {`import { Resolvers } from "./generated/resolvers";
import { User } from "@/model/user";

type Context = {};

const resolvers: Resolvers<Context> = {
  Query: {
    me: async () => {
      // Returns the current user.
      return new User("12345", "uhyo");
    },
  },
  User: {
    // \`user\` is an instance of User class
    id: (user) => user.id,
    name: (user) => user.name,
    email: (user) => {
      return user.getEmail();
    },
    posts: (user) => {
      return user.getPosts();
    },
  },
  Post: {
    // ...
  },
};`}
      </Highlight>
      <p>
        Under this mode, you need to implement all field resolvers for the type.
      </p>

      <h3 id="using-the-server-graphql-schema-file">
        Using the server GraphQL schema file
      </h3>
      <p>
        A smart reader might remember that nitrogql 1.1 also generates a{" "}
        <strong>server GraphQL schema file</strong>. Actually this file is
        simple; it just exports the GraphQL schema as a string. For example:
      </p>
      <Highlight language="typescript">
        {`// generated by nitrogql
export const schema = \`
type Query {
  me: User!
}

// ...
\`;`}
      </Highlight>
      <p>
        Even if you have multiple <code>.graphql</code> files to form a schema,
        they are concatenated and exported as a single string. This reduces the
        burden of manually loading all those files when initializing a GraphQL
        server.
      </p>
      <p>
        Also, this works as another layer of safety by ensuring that the schema
        you use at runtime is the same as the schema you used for generating
        type definitions. <i>One configuration to rule them all</i> is a great
        principle for reducing the chance of human errors.
      </p>
      <p>
        You can use this file to initialize a GraphQL server. For example, with{" "}
        <a
          href="https://www.apollographql.com/docs/apollo-server/"
          target="_blank"
        >
          Apollo Server
        </a>
        :
      </p>
      <Highlight language="typescript">
        {`import { ApolloServer } from "@apollo/server";
import { schema } from "./generated/server-graphql";
import { Resolvers } from "./generated/resolvers";

const resolvers: Resolvers = { /* ... */ };

const server = new ApolloServer({
  typeDefs: schema,
  resolvers,
});`}
      </Highlight>

      <h4 id="schema-cleanup">Schema cleanup</h4>
      <p>
        In fact, the server GraphQL schema file is not just a concatenation of
        all the <code>.graphql</code> files. It&apos;s also processed to remove
        all usage of the <code>@model</code> directive.
      </p>
      <p>
        We knew that some of you would complain that they don&apos;t want to{" "}
        <i>pollute</i> their schema with directives that don&apos;t affect
        runtime behavior at all.
      </p>
      <p>
        Our position on this is that we utilize the schema as the{" "}
        <i>single source of truth</i> for both at runtime and at compile time.
        If you need to annotate any GraphQL type with some information for
        compile time, we prefer to do that in the schema.
      </p>
      <p>
        Anyway, since it isn&apos;t bad to remove compile-time-only directives
        from runtime code, we do that. The server GraphQL schema file is the
        result of this process.
      </p>

      <h3 id="the-plugin">
        The <code>nitrogql:model</code> plugin
      </h3>
      <p>
        In fact, all those <code>@model</code> story is implemented as a
        built-in plugin called <code>nitrogql:model</code>. You must enable this
        plugin in order to use the <code>@model</code> directive. As mentioned
        in the beginning of this article, this is done by adding the plugin to
        the <code>plugins</code> option.
      </p>
      <Highlight language="yaml">
        {`schema: ./schema/*.graphql
documents: ./src/**/*.graphql
extensions:
  nitrogql:
    plugins:
      - "nitrogql:model"
    # ...`}
      </Highlight>
      <p>
        We felt that custom directives are too opinionated to be enabled by
        default. That&apos;s why we made it an opt-in feature.
      </p>
      <p>
        However, resolvers type generation is almost unusable without the
        plugin. The default behavior is that the model object for each type
        includes <em>all</em> the fields in the type, and you still need to
        implement resolvers for all the fields. This is still type-safe but not
        very practical.
      </p>
      <p>
        Type safety is a <em>very</em> important goal for nitrogql. Any
        combination of options should be type-safe, even if it&apos;s not
        practically usable.
      </p>
      <p>
        To make resolver development practical while maintaining type safety,
        the developer must be able to specify which fields are included in the
        model object. This is why we introduced the <code>@model</code>{" "}
        directive through the plugin.
      </p>
      <p>
        Maybe just for fun, let me share other options of default behavior
        without the plugin:
      </p>
      <p>
        <b>All fields in the model, all resolvers must be defined.</b> This is
        the chosen option.
      </p>
      <p>
        <b>
          All fields in the model, no resolvers need to be defined (all fields
          use the default resolver).
        </b>{" "}
        This is actually still type safe. However, this is likely to guide
        GraphQL beginners to wrong direction by making them think that they
        don&apos;t need to implement resolvers at all. This is the opposite of
        how GraphQL is supposed to be used. We don&apos;t want to encourage
        beginners to do that.
      </p>
      <p>
        <b>Fields in the model are optional, all resolvers must be defined.</b>{" "}
        This is <em>kind of </em> safe because as long as all resolvers are
        defined, it never fails to return needed data. However, this setting
        would force you to write a lot of boring boilerplate code. Proper type
        definitions should be able to improve the developer experience than
        this.
      </p>
      <p>
        <b>Fields in the model are optional and resolvers are also optional.</b>{" "}
        This is the default behavior of GraphQL Code Generator. But we
        don&apos;t do this because it&apos;s not type safe. If you omit a field
        from a resolver return value and also don&apos;t implement an explicit
        resolver for it, it will be a runtime error.
      </p>

      <h3 id="whats-next">What&apos;s Next?</h3>
      <p>
        In fact our roadmap is empty now. This doesn&apos;t mean that we&apos;re
        done with nitrogql. We&apos;re considering some ideas for the next
        release, but we haven&apos;t decided yet.
      </p>
      <p>
        If you have any ideas or requests, please let us know on{" "}
        <a href="https://github.com/uhyo/nitrogql" target="_blank">
          GitHub
        </a>
        . We&apos;re waiting for your feedback!
      </p>

      <h3 id="conclusion">Conclusion</h3>
      <p>
        nitrogql 1.1 is a big step towards the goal of type safety on both the
        client and the server. Now you can get type safety on both sides using
        the same GraphQL schema. We hope that this will make GraphQL development
        more enjoyable.
      </p>
      <p>
        In <Link href="/blog/release-1.0">the last article</Link>, we said that
        GraphQL Code Generator&apos;s resolver type generation is not type-safe
        by default. Actually, the <code>mappers</code> option is the only way to
        get type safety in a reasonable way with that tool.
      </p>
      <p>
        While nitrogql supports the same way of defining models (by applying the{" "}
        <code>@model</code> directive to the entire type), we have another way
        applying the directive to each field. We like this way better because
        this is easer to use and it doesn&apos;t require additional type
        definitions external to resolver implementations.
      </p>
      <p>
        In conclusion, this release introduces something unfamiliar to you, but
        we believe that it is nice enough. We hope that you will like it too.
      </p>

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
