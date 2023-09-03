import { Highlight } from "@/app/_utils/Highlight";
import { ArticleMetadata } from "./_meta";
import Link from "next/link";
import { Hint } from "@/app/_utils/Hint";

export const blogPostRelease1_0: ArticleMetadata = {
  slug: "release-1.0",
  title: "nitrogql 1.0 release",
  shortDescription: `Today, we are happy to announce the first stable release of nitrogql!
nitrogql is a toolchain for using GraphQL from TypeScript projects.
In this post, we will go over the main features of nitrogql and how to get started with it.`,
  publishDate: new Date("2023-09-01T00:00Z"),
  render,
};

function render() {
  return (
    <>
      <p>
        Today, we are happy to announce{" "}
        <strong>the first stable release</strong> of nitrogql!
      </p>
      <p>
        <b>nitrogql</b> is a toolchain for using GraphQL in TypeScript projects.
        In this post, we will go over the main features of nitrogql and how to
        get started with it.
      </p>

      <h3 id="what-is-nitrogql">What is nitrogql?</h3>
      <p>
        Currently, nitrogql has two main features:{" "}
        <strong>code generation</strong> and <strong>static checking</strong>.
      </p>

      <h4 id="code-generation">Code generation</h4>
      <p>
        <b>Code generation</b> is the process of generating TypeScript code from
        GraphQL code (both schema and operations). This is useful because it
        allows you to use GraphQL operations in your TypeScript code with{" "}
        <strong>type safety</strong> and without having to write any boilerplate
        code.
      </p>
      <p>
        This is known as the <b>schema-first</b> approach to GraphQL. In this
        approach, you write your GraphQL schema first, and then you generate
        code from it. This is the opposite of the code-first approach, where you
        write your code first, and then you generate a schema from it.
      </p>
      <p>For example, if you have a GraphQL operation like this:</p>
      <Highlight language="graphql">
        {`# getPosts.graphql
query getPosts {
  posts {
    id
    title
  }
}`}
      </Highlight>
      <p>
        Then you can use it in your TypeScript code like this (after running{" "}
        <code>nitrogql generate</code>):
      </p>
      <Highlight language="typescript">
        {`import { useQuery } from "@apollo/client";
import getPostsQuery from "./getPosts.graphql";

export const Posts: React.FC = () => {
  const { data } = useQuery(getPostsQuery);
  return (
    <ul>
      {data.posts.map((post) => (
        <li key={post.id}>{post.title}</li>
      ))}
    </ul>
  );
};`}
      </Highlight>
      <p>
        Notably, nitrogql allows you to import GraphQL operations from{" "}
        <code>.graphql</code> files. Thanks to{" "}
        <a
          href="https://github.com/dotansimha/graphql-typed-document-node"
          target="_blank"
        >
          TypedDocumentNode
        </a>
        , the imported operation knows the shape of the data it returns. This is
        why the type of <code>data</code> is also derived from the GraphQL
        operation, namely as{" "}
        <code>{"{ posts: { id: string; title: string; }[]; }"}</code>.
      </p>
      <p>
        For maximum safety, nitrogql provides a webpack loader and a Rollup
        plugin that let you import <code>.graphql</code> files directly from
        your TypeScript code as shown above. This way, nitrogql can be in charge
        of both runtime and type-level behavior of your <code>.graphql</code>{" "}
        files so that nitrogql can ensure that they are always in sync.
      </p>

      <h4 id="static-checking">Static checking</h4>
      <p>
        While code generation provides TypeScript-level safety, it is still
        possible to make mistakes in your GraphQL code. For example, you might
        try to query a field that does not exist in the schema, or you might
        forget to pass a required argument to a field. These in-schema errors
        and schema-operation-mismatch errors are not covered by above code
        generation because it is all about using known-good GraphQL code in
        TypeScript.
      </p>
      <p>
        This is where <b>static checking</b> comes in. Static checking is the
        process of checking your GraphQL code for errors before you generate any
        TypeScript code from it. This GraphQL-level safety guards you against
        seeing runtime errors in your application that comes from mistakes in
        your GraphQL code.
      </p>
      <p>
        The following is an example of a GraphQL-level error that nitrogql can
        catch:
      </p>
      <Highlight language="graphql">
        {`# getPosts.graphql
query getPosts {
  posts { # forgot to pass an argument
    id
    tite # typo here
  }
}`}
      </Highlight>
      <p>
        When you run <code>nitrogql check</code>, nitrogql will report the
        following error:
      </p>
      <Highlight language="text">
        {`query getPosts {
  posts {
  ^
  Required argument 'filter' is not specified

    # forgot to pass an argument
    id
    tite # typo here
    ^
    Field 'tite' is not found on type 'Post'
  }
}`}
      </Highlight>

      <h3 id="why-nitrogql">Why nitrogql?</h3>
      <p>
        You might already be familiar with other tools that provide similar
        features to nitrogql. Particularly,{" "}
        <a href="https://graphql-code-generator.com/" target="_blank">
          GraphQL Code Generator
        </a>{" "}
        is a popular tool that provides code generation for GraphQL.
      </p>
      <p>
        Then, why did we create nitrogql instead of using GraphQL Code
        Generator? There are a few reasons:
      </p>
      <ul>
        <li>We want source maps.</li>
        <li>
          We like the <code>near-operation-file</code> preset of GraphQL Code
          Generator.
        </li>
        <li>We want maximum safety as a default.</li>
      </ul>
      <p>Let&apos;s go over each of these reasons in detail.</p>

      <h4 id="source-maps">Source maps</h4>
      <p>
        Source maps are a very helpful feature when one is using code
        generation. They allow you to see the original GraphQL code that
        generated a particular TypeScript code. This is very useful when you use
        a Go to Definition feature of your editor. For example, if you use the
        Go to Definition feature on <code>getPostsQuery</code> in the following
        code:
      </p>
      <Highlight language="typescript">
        {`import { useQuery } from "@apollo/client";
import getPostsQuery from "./getPosts.graphql";

export const Posts: React.FC = () => {
  const { data } = useQuery(getPostsQuery);
  return (
    <ul>
      {data.posts.map((post) => (
        <li key={post.id}>{post.title}</li>
      ))}
    </ul>
  );
};`}
      </Highlight>
      <p>
        Then, you will be taken to the <code>getPosts.graphql</code> file
        instead of the generated <code>getPostsQuery.d.graphql.ts</code> file.
        This is because nitrogql generates source maps that point to the
        original GraphQL code.
      </p>
      <p>
        GraphQL Code Generator does not support source maps. Actually, I tried
        to add source map support to GraphQL Code Generator before creating
        nitrogql, but I found that the architecture of GraphQL Code Generator is
        making it very difficult to add source map support. You know, source map
        support must be built into the core of a code generation tool. Adding it
        later is seriously hard. So, I decided to create a new tool that has
        source map support from the beginning.
      </p>

      <h4 id="near-operation-file">near-operation-file preset</h4>
      <p>
        The <code>near-operation-file</code> preset <i>was</i> the recommended
        way to use GraphQL Code Generator. It is a very nice feature that allows
        you to keep your GraphQL code and TypeScript code separate from each
        other.
      </p>
      <p>
        However, while it is still fully supported, the{" "}
        <code>near-operation-file</code> preset is no longer recommended as
        GraphQL Code Generator&apos;s default. Instead, the{" "}
        <code>client-preset</code> is now the default.
      </p>
      <p>
        I don&apos;t really like the new default because it requires you to put
        GraphQL code and TypeScript code in the same file. Especially, it
        examines GraphQL code in your TypeScript code and generates types from
        it. Then, the generated code affects that TypeScript code. I don&apos;t
        quite like this circular process.
      </p>
      <p>
        nitrogql is the opposite of this. It requires you to put GraphQL code in{" "}
        <code>.graphql</code> files, completely separated from TypeScript code.
        This is similar to what the <code>near-operation-file</code> preset
        does. nitrogql will keep going with this approach. If you like the{" "}
        <code>near-operation-file</code> preset, you will like nitrogql too.
      </p>

      <h4 id="maximum-safety-as-a-default">Maximum safety as a default</h4>
      <p>
        GraphQL Code Generator is a very flexible tool. It allows you to
        configure many things. What is unfortunate is that some of the options
        are footguns that decrease type safety.
      </p>
      <p>
        For example, the types for custom scalars are <code>any</code> by
        default. Unless you set{" "}
        <a
          href="https://the-guild.dev/graphql/codegen/plugins/typescript/typescript#strictscalars"
          target="_blank"
        >
          <code>strictScalars</code>
        </a>{" "}
        to <code>true</code>, GraphQL Code Generator does&apos;t even warn you
        about this.
      </p>
      <p>
        Also, if you use GraphQL Code Generator for generating resolver type
        definitions, it is very easy to forget to define one resolver and
        another, and then you will see a runtime error. I wrote in details about
        this in{" "}
        <a
          href="https://zenn.dev/babel/articles/graphql-typing-for-babel"
          target="_blank"
        >
          another article (in Japanese)
        </a>
        .
      </p>
      <p>
        nitrogql is designed to be as safe as possible by default. For example,
        the types for custom scalars must be explicitly defined. Failing to do
        so will result in a compile error. There is no way to disable this
        behavior.
      </p>
      <p>
        While nitrogql still has some configuration options, they are for
        different use cases. Loosening type safety is not one of them.
      </p>

      <h3 id="getting-started">Getting started</h3>
      <p>
        While we have a{" "}
        <Link href="/guides/getting-started">Getting Started</Link> page in this
        site, let&apos;s go over the basics here too.
      </p>
      <p>nitrogql is a CLI tool. You can install it with:</p>
      <Highlight language="bash">
        {`npm install --save-dev @nitrogql/cli @graphql-typed-document-node/core`}
      </Highlight>
      <Hint>
        Note: <code>@graphql-typed-document-node/core</code> is required because
        it is depended by generated code.
      </Hint>
      <p>
        Then you need to create a configuration file named{" "}
        <code>graphql.config.yaml</code> in the root of your project. The
        following is an example of a configuration file:
      </p>
      <Highlight language="yaml">
        {`schema: ./schema/*.graphql
documents: ./src/**/*.graphql
extensions:
  nitrogql:
    generate:
      schemaOutput: ./src/generated/schema.d.ts
`}
      </Highlight>
      <p>
        This configuration file tells nitrogql to look for GraphQL schema files
        in <code>./schema</code> and GraphQL operation files in{" "}
        <code>./src</code>. Note that they have different syntaxes and cannot be
        mixed.
      </p>
      <p>
        Then, you can run <code>nitrogql generate</code> to generate TypeScript
        code from your GraphQL code. This command will generate{" "}
        <code>.d.graphql.ts</code> files next to your <code>.graphql</code>{" "}
        files. With the help of these generated files, you can import{" "}
        <code>.graphql</code> files from your TypeScript code in a type-safe
        manner.
      </p>
      <p>
        You can also run <code>nitrogql check</code> to check your GraphQL code
        for errors.
      </p>
      <p>
        For more information, please refer to the{" "}
        <Link href="/configuration/options">Configuration Options</Link> page.
      </p>

      <h3 id="migrating-from-graphql-code-generator">
        Migrating from GraphQL Code Generator
      </h3>
      <p>
        If you are already using GraphQL Code Generator, you might want to
        migrate to nitrogql. Migration will be a tough process because they are
        not fully compatible. However, we have a detailed{" "}
        <Link href="/guides/migrating-from-graphql-codegen">
          migration guide
        </Link>{" "}
        for you.
      </p>

      <h3 id="check-only-usage">Check-only usage</h3>
      <p>
        If you are interested in using nitrogql but can&apos;t fully switch from
        GraphQL Code Generator yet, you can use nitrogql only as a linter. Just
        run <code>nitrogql check</code> in CI to check your GraphQL code for
        errors. This way, you can get the benefits of static checking without
        having to migrate to nitrogql completely.
      </p>
      <p>
        Configuration for check-only usage is very simple. You just need to
        specify the <code>schema</code> and <code>documents</code> option in
        your configuration file.
      </p>
      <Highlight language="yaml">
        {`schema: ./schema/*.graphql
documents: ./src/**/*.graphql`}
      </Highlight>
      <p>
        Then run <code>nitrogql check</code> in CI. If there are any errors in
        your GraphQL code, it will exit with a non-zero exit code. That&apos;s
        it!
      </p>
      <p>
        Please note, however, that nitrogql&apos;s static checking follows the
        GraphQL specification strictly. This means that it will report errors
        for code that may actually work with GraphQL Code Generator. For
        example, if you define a fragment, nitrogql does not allow you to refer
        to it from other files. Fragments are local to the file where they are
        defined. This means that nitrogql is (currently) not compatible with
        some fragment collocation techniques used in the community. We are still
        investigating how to support them.
      </p>

      <h3 id="whats-next">What&apos;s next?</h3>
      <p>
        nitrogql is still in its early days. Currently it has only two features
        mentioned above that are considered stable from today. We will keep
        improving them, but we also have many ideas for the future. Here are
        some of them.
      </p>
      <p>
        <strong>Generating resolver types.</strong> Currently, nitrogql only
        generates types for GraphQL operations. This means that nitrogql helps
        you with the client-side of GraphQL, but not the server-side. We will
        work on generating types for resolvers too, so that you can get the
        benefits of type safety on both the client-side and the server-side.
      </p>
      <p>
        <strong>Plugin system.</strong> Currently, nitrogql has a small set of
        configuration options. Some of the feature we want to add are kind of
        opinionated, so we don&apos;t want to add them as configuration options.
        Instead, we want to add a plugin system so that various use cases can be
        supported by nitrogql.
      </p>

      <h3 id="conclusion">Conclusion</h3>
      <p>
        nitrogql is a toolchain for using GraphQL from TypeScript projects. It
        provides code generation and static checking. If you liked GraphQL Code
        Generator&apos;s <code>near-operation-file</code> preset, nitrogql is a
        nice alternative for you with source map support and more safety.
      </p>
      <p>
        Visit{" "}
        <a href="https://github.com/uhyo/nitrogql" target="_blank">
          GitHub
        </a>{" "}
        for more information.
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
