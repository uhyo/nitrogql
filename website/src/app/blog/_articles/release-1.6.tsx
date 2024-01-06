import { Highlight } from "@/app/_utils/Highlight";
import { ArticleMetadata } from "./_meta";
import Link from "next/link";
import { Hint } from "@/app/_utils/Hint";

export const blogPostRelease1_6: ArticleMetadata = {
  slug: "release-1.6",
  title: "nitrogql 1.6 release: improved treatment of scalar types",
  shortDescription: `In nitrogql 1.6, treatment of scalar types has been improved.
Now each GraphQL scalar type can have different TypeScript type in different situations.`,
  publishDate: new Date("2023-01-06T00:00Z"),
  render,
};

function render() {
  return (
    <>
      <p>
        Today, we are happy to announce release of <strong>nitrogql 1.6</strong>
        !
      </p>
      <p>
        <b>nitrogql</b> is a toolchain for using GraphQL in TypeScript projects.
        In 1.6, treatment of scalar types has been improved. Now each GraphQL
        scalar type can have different TypeScript type in different situations.
      </p>

      <h3 id="problem-of-the-id-type">
        Problem of the <code>ID</code> type
      </h3>
      <p>
        Let&apos;s start the story with the <code>ID</code> type.{" "}
        <code>ID</code> is a scalar type that is built into GraphQL. It is used
        to represent unique identifiers.
      </p>
      <p>
        What is special about <code>ID</code> is that integers, as well as
        strings, can be used to represent <code>ID</code> values. However, an{" "}
        <code>ID</code> value is always serialized as a string. For example,
        assume the following object is returned by a GraphQL resolver:
      </p>
      <Highlight language="typescript">{`{
  id: 123, // ID
  name: "Alice", // String
}`}</Highlight>
      <p>Then, the client will observe the following JSON object:</p>
      <Highlight language="typescript">{`{
  "id": "123",
  "name": "Alice"
}`}</Highlight>
      <p>
        Type definition generators have to deal with this asymmetry, but
        nitrogql did not until this release. In 1.5, <code>ID</code> was always
        treated as <code>string</code> in TypeScript. This was too restrictive
        than necessary.
      </p>
      <p>
        In 1.6, <code>ID</code> has the following default mapping:
      </p>
      <Highlight language="yaml">{`ID:
  send: string | number
  receive: string`}</Highlight>
      <p>
        This post will explain what this means and how to customize scalar type
        mappings in nitrogql 1.6.
      </p>

      <h3 id="four-different-usage-of-graphql-types">
        Four different usage of GraphQL types
      </h3>
      <p>
        In a GraphQL application written in TypeScript, each GraphQL type can be
        used in four different ways, two of which are on the server side and the
        other two are on the client side.
      </p>

      <h4 id="resolver-input-position">Resolver input position</h4>
      <p>
        The first usage is in the input position of a resolver. Consider the
        following GraphQL schema:
      </p>
      <Highlight language="graphql">
        {`type Query {
  user(id: ID!): User!
}`}
      </Highlight>
      <p>
        Then, the implementation of the <code>user</code> resolver will look
        like this:
      </p>
      <Highlight language="typescript">
        {`const userResolver: Resolvers<Context>["Query"]["user"] = async (
  _,
  { id },
  // ^ \`id\` used in the resolver input position
) => {
  // ...
}`}
      </Highlight>
      <p>
        In the above code, <code>id</code> is used in the input position of the
        resolver. In this case the type of <code>id</code> is{" "}
        <code>string</code> regardless of whether the client sends an integer or
        a string. This is because the GraphQL server applies coercion to the
        input values before the resolver is called.
      </p>

      <h4 id="resolver-output-position">Resolver output position</h4>
      <p>
        The second usage is in the output position of a resolver. It is
        illustrated by the following example:
      </p>
      <Highlight language="typescript">
        {`const userResolver: Resolvers<Context>["Query"]["user"] = async (
  _,
  { id },
) => {
  // ...
  return {
    id: user.id,
  // ^ \`id\` used in the resolver output position
    name: user.name,
  };
}`}
      </Highlight>
      <p>
        Assuming that the <code>id</code> field of the <code>User</code> type is{" "}
        <code>ID</code>, the type of <code>user.id</code> can be{" "}
        <code>string | number</code>. This value is then serialized as a string
        when it is sent to the client.
      </p>

      <h4 id="operation-input-position">Operation input position</h4>
      <p>
        The third usage is in the input position of an operation. Assume you
        want to run the following GraphQL operation:
      </p>
      <Highlight language="graphql">
        {`query GetUser($id: ID!) {
  user(id: $id) {
    id
    name
  }
}`}
      </Highlight>
      <p>Then, typical client-side code will look like this:</p>
      <Highlight language="typescript">
        {`const { data } = await client.query({
  query: GetUserQuery,
  variables: {
    id: "123",
  // ^ \`id\` used in the operation input position
  },
});`}
      </Highlight>
      <p>
        In this case, you can pass either a string or a number to the{" "}
        <code>id</code> variable. The type of <code>id</code> is{" "}
        <code>string | number</code>. This value is then sent to the server
        as-is (without coercing it to a string).
      </p>
      <Hint>
        🕵️‍♂️ Note that the client does not know about the schema in a normal
        setting. Therefore, the client cannot apply coercion to the variable
        values based on their types.
      </Hint>

      <h4 id="operation-output-position">Operation output position</h4>
      <p>
        The fourth usage is in the output position of an operation. It is
        illustrated by the following example:
      </p>
      <Highlight language="typescript">
        {`const { data } = await client.query({
  query: GetUserQuery,
});
// ...
const id = data.user.id;
//    ^ \`id\` used in the operation output position`}
      </Highlight>
      <p>
        In this case, the type of <code>id</code> is <code>string</code>. This
        is because the value is always serialized as a string before it is sent
        to the client.
      </p>

      <h4 id="summary-of-the-four-usages">Summary of the four usages</h4>
      <p>To summarize, each GraphQL type can be used in four different ways:</p>
      <table>
        <thead>
          <tr>
            <th>Usage</th>
            <th>Location</th>
            <th>
              TypeScript type (<code>ID</code>)
            </th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td>Resolver input</td>
            <td>Server</td>
            <td>
              <code>string</code>
            </td>
          </tr>
          <tr>
            <td>Resolver output</td>
            <td>Server</td>
            <td>
              <code>string | number</code>
            </td>
          </tr>
          <tr>
            <td>Operation input</td>
            <td>Client</td>
            <td>
              <code>string | number</code>
            </td>
          </tr>
          <tr>
            <td>Operation output</td>
            <td>Client</td>
            <td>
              <code>string</code>
            </td>
          </tr>
        </tbody>
      </table>
      <p>
        Notice that you cannot categorize these as server/client or
        input/output, looking at the <code>ID</code> example.
      </p>
      <p>
        Instead, nitrogql adopted the <strong>send/receive</strong> terminology
        to categorize these usage into two groups. The <strong>send</strong>{" "}
        group contains the “resolver output” and the “operation input” usages.
        The <strong>receive</strong> group contains the “resolver input” and the
        “operation output” usages.
      </p>
      <p>
        These terminologies come from the fact that the “send” group is used
        where a value is being <em>sent</em> to the other side, and the
        “receive” group is used where a value is being <em>received</em> from
        the other side. Existing GraphQL servers behave such that values in the
        “receive” group are already coerced, while values in the “send” group
        are not.
      </p>
      <p>
        At least this is the best fit for the <code>ID</code> type, which is why
        we adopted this terminology in nitrogql.
      </p>

      <h3 id="customizing-scalar-type-mappings">
        Customizing scalar type mappings
      </h3>
      <p>
        In nitrogql 1.6, you can specify different TypeScript types for each
        usage of a GraphQL scalar type.
      </p>
      <p>
        nitrogql now supports three forms to specify scalar type mappings:
        <b>single</b>, <b>send/receive</b> and <b>separate</b>.
      </p>

      <h4 id="single-form">Single form</h4>
      <p>
        The <b>single</b> form is the simplest form. It is the same as the
        scalar type mapping in nitrogql 1.5. You can specify a single TypeScript
        type for all usages of a GraphQL scalar type. For example, the following
        configuration specifies that <code>String</code> is always treated as{" "}
        <code>string</code>:
      </p>
      <Highlight language="yaml">{`String: string`}</Highlight>
      <p>
        All built-in scalar types except <code>ID</code> are configured in this
        form by default.
      </p>

      <h4 id="send-receive-form">Send/receive form</h4>
      <p>
        The <b>send/receive</b> form allows you to specify different TypeScript
        types for the “send” group and the “receive” group. For example, the
        following configuration specifies that <code>ID</code> is treated as{" "}
        <code>string | number</code> in the “send” group and as{" "}
        <code>string</code> in the “receive” group:
      </p>
      <Highlight language="yaml">{`ID:
  send: string | number
  receive: string`}</Highlight>
      <p>
        This is the default configuration for <code>ID</code> in nitrogql 1.6.
      </p>

      <h4 id="separate-form">Separate form</h4>
      <p>
        The <b>separate</b> form allows you to specify different TypeScript
        types for each usage. For example, the mapping for <code>ID</code> could
        be specified as follows in this form:
      </p>
      <Highlight language="yaml">{`ID:
  resolverInput: string
  resolverOutput: string | number
  operationInput: string | number
  operationOutput: string`}</Highlight>
      <p>
        This form exists for completeness, but we have not found a use case for
        it yet. If you have a use case for this form, please let us know!
      </p>

      <h3 id="notes-on-graphql-code-generator-compatibility">
        Notes on GraphQL code generator compatibility
      </h3>
      <p>
        If you are using GraphQL code generator, you might know that it has a
        similar feature which allows you to specify different TypeScript types
        for different situations. Namely, it allows you to specify an{" "}
        <code>input</code>
        type and an <code>output</code> type for each GraphQL scalar type. For
        example, the mapping for <code>ID</code> could be specified as follows
        in GraphQL code generator:
      </p>
      <Highlight language="yaml">{`# GraphQL code generator config
ID:
  input: string
  output: string | number`}</Highlight>
      <p>
        However, this input/output semantics is different from the send/receive
        semantics in nitrogql. They are summarized in the following table:
      </p>
      <table>
        <thead>
          <tr>
            <th>Usage</th>
            <th>Location</th>
            <th>GraphQL code generator</th>
            <th>nitrogql</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td>resolverInput</td>
            <td>server</td>
            <td>input</td>
            <td>receive</td>
          </tr>
          <tr>
            <td>resolverOutput</td>
            <td>server</td>
            <td>output</td>
            <td>send</td>
          </tr>
          <tr>
            <td>operationInput</td>
            <td>client</td>
            <td>input</td>
            <td>send</td>
          </tr>
          <tr>
            <td>operationOutput</td>
            <td>client</td>
            <td>output</td>
            <td>receive</td>
          </tr>
        </tbody>
      </table>
      <p>
        This divergence is intentional. Our investigation shows that the
        send/receive semantics better reflects the actual behavior of GraphQL
        servers. We avoided using the same terminology as GraphQL code generator
        (input/output) and chose to invent our own (send/receive) instead.
      </p>

      <h3 id="conclusion">Conclusion</h3>
      <p>
        nitrogql 1.6 allows you to specify different TypeScript types for
        different usages of a GraphQL scalar type. This allows you to use the{" "}
        <code>ID</code> type in a more convenient way.
      </p>
      <p>
        While GraphQL Code Generator already has a similar feature, the
        semantics is different. We chose to invent our own terminology to better
        reflect the reality.
      </p>

      <hr />

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
