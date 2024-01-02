import Link from "next/link";
import { Hint } from "@/app/_utils/Hint";
import { Highlight } from "@/app/_utils/Highlight";
import { Toc } from "../../_toc";
import { Breadcrumb } from "@/app/_utils/Breadcrumb";
import { ogp } from "@/app/_utils/metadata";

export const metadata = ogp({
  title: "Configuring Scalar Types",
});

export default function ConfiguringScalarTypes() {
  return (
    <Toc>
      <main>
        <Breadcrumb
          parents={[{ label: "Configuration", href: "/configuration" }]}
          current="Configuring Scalar Types"
        />
        <h2>Configuring Scalar Types</h2>
        <p>
          nitrogql supports configuring the mapping between GraphQL scalar types
          and TypeScript types. This is useful when you want to use custom
          scalar types in your GraphQL schema. The mapping is configured with
          the{" "}
          <Link href="/configuration/options#generate.type.scalarTypes">
            <code>generate.type.scalarTypes</code>
          </Link>{" "}
          option.
        </p>
        <p>
          This page explains how to configure scalar types for different
          scenarios.
        </p>

        <Hint>
          😶‍🌫️ This page does not cover the{" "}
          <Link href="/references/plugin-graphql-scalars">
            nitrogql:graphql-scalars-plugin
          </Link>
          . If you want to use custom scalar from GraphQL Scalars, you should
          read that page instead.
        </Hint>

        <h3 id="four-different-situations">Four Different Situations</h3>
        <p>
          nitogql supports specifying different TypeScript types for one GraphQL
          scalar type. They are used in different situations:
        </p>
        <table>
          <thead>
            <tr>
              <th>Name</th>
              <th>Location</th>
              <th>Situation</th>
              <th>
                Example (<code>ID</code>)
              </th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td>resolverInput</td>
              <td>server</td>
              <td>as an argument of a resolver</td>
              <td>
                <code>string</code>
              </td>
            </tr>
            <tr>
              <td>resolverOutput</td>
              <td>server</td>
              <td>as a return value of a resolver</td>
              <td>
                <code>string | number</code>
              </td>
            </tr>
            <tr>
              <td>operationInput</td>
              <td>client</td>
              <td>as an argument passed to an operation</td>
              <td>
                <code>string | number</code>
              </td>
            </tr>
            <tr>
              <td>operationOutput</td>
              <td>client</td>
              <td>as a return value of an operation</td>
              <td>
                <code>string</code>
              </td>
            </tr>
          </tbody>
        </table>
        <p>
          The first two types are used in the server, and the last two types are
          used in the client.
        </p>
        <p>
          An example of a scalar type that uses different types in different
          situations is the <code>ID</code> scalar type. According to the{" "}
          <a href="https://spec.graphql.org/October2021/#sec-ID">
            GraphQL spec
          </a>
          , an <code>ID</code> value is always serialized as a string. However,
          it can be represented as a string or an integer in the server. Above
          table shows how the <code>ID</code> scalar type is configured for
          different situations by default.
        </p>

        <h4 id="resolverinput">resolverInput</h4>
        <p>
          The <code>resolverInput</code> type is used in a GraphQL server when
          executing a resolver. Whenever a resolver receives an argument of the
          scalar type, it will be of this type.
        </p>
        <p>
          <b>Example:</b>
        </p>
        <Highlight language="graphql">
          {`type Query {
  getUser(id: ID!): User!
}`}
        </Highlight>
        <Highlight language="ts">
          {`const getUser: Resolvers<Context>["Query"]["getUser"] = async (
  _,
  { id },
) => {
  // \`id\` is typed as its \`resolverInput\` variant (string)
  // ...
};`}
        </Highlight>

        <h4 id="resolveroutput">resolverOutput</h4>
        <p>
          The <code>resolverOutput</code> type is used in a GraphQL server when
          executing a resolver. Whenever a resolver returns a value of the
          scalar type, it will be of this type.
        </p>
        <p>
          <b>Example:</b>
        </p>
        <Highlight language="graphql">
          {`type Query {
  me: User!
}`}
        </Highlight>
        <Highlight language="ts">
          {`const me: Resolvers<Context>["Query"]["me"] = async () => {
  return {
    // \`id\` is typed as its \`resolverOutput\` variant (string | number)
    id: 1,
    // ...
  };
};`}
        </Highlight>

        <h4 id="operationinput">operationInput</h4>
        <p>
          The <code>operationInput</code> type is used in a GraphQL client when
          executing an operation. Whenever an operation requires an argument of
          the scalar type, you need to pass a value of this type.
        </p>
        <p>
          <b>Example:</b>
        </p>
        <Highlight language="graphql">
          {`query GetUser($id: ID!) {
  getUser(id: $id) {
    # ...
  }
}`}
        </Highlight>
        <Highlight language="ts">
          {`const result = await yourClient.query({
  query: GetUserQuery,
  variables: {
    // \`id\` is typed as its \`operationInput\` variant (string | number)
    id: 1,
  },
});`}
        </Highlight>

        <h4 id="operationoutput">operationOutput</h4>
        <p>
          The <code>operationOutput</code> type is used in a GraphQL client when
          executing an operation. Whenever an operation returns a value of the
          scalar type, it will be of this type.
        </p>
        <p>
          <b>Example:</b>
        </p>
        <Highlight language="graphql">
          {`query Me {
  me {
    id
    # ...
  }
}`}
        </Highlight>
        <Highlight language="ts">
          {`const result = await yourClient.query({
  query: MeQuery,
});

// \`id\` is typed as its \`operationOutput\` variant (string)
const id = result.data.me.id;`}
        </Highlight>

        <h3 id="specifying-scalartypes-in-configuration">
          Specifying <code>scalarTypes</code> in Configuration
        </h3>
        <p>
          When you configure the TypeScript types for a scalar type, you can use
          one of the following three forms. In addition to one that specifies
          all four types, there are two convenient shorthand forms. Actually,
          you rarely need to specify all four types, so the shorthand forms are
          usually enough.
        </p>

        <h4 id="single-form">Single Form</h4>
        <p>
          The easiest way to configure a scalar type is to use the single form.
          This form specifies to use one TypeScript type for all four
          situations.
        </p>
        <p>
          <b>Example:</b>
        </p>
        <Highlight language="yaml">
          {`scalarTypes:
  String: string`}
        </Highlight>
        <p>
          With this setting, the <code>String</code> scalar type will be mapped
          to the <code>string</code> type in all four situations.
        </p>
        <p>
          Fundamental scalar types such as <code>String</code>, <code>Int</code>
          , <code>Float</code> and <code>Boolean</code> are configured with the
          single form by default. These types always map to the same TypeScript
          types (i.e. <code>string</code>, <code>number</code>,{" "}
          <code>number</code> and <code>boolean</code>, respectively).
        </p>

        <h4 id="send-receive-form">Send/Receive Form</h4>
        <p>
          The send/receive form is another shorthand form that specifies two
          TypeScript types for a scalar type: one for <i>send</i> situations,
          and the other for <i>receive</i> situations. They map to the four
          situations as follows:
        </p>
        <table>
          <thead>
            <tr>
              <th>Name</th>
              <th>Send/Receive</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td>resolverInput</td>
              <td>receive</td>
            </tr>
            <tr>
              <td>resolverOutput</td>
              <td>send</td>
            </tr>
            <tr>
              <td>operationInput</td>
              <td>send</td>
            </tr>
            <tr>
              <td>operationOutput</td>
              <td>receive</td>
            </tr>
          </tbody>
        </table>
        <p>
          Among the built-in scalar types, the <code>ID</code> scalar type is
          configured with the send/receive form by default.
        </p>
        <Highlight language="yaml">
          {`scalarTypes:
  ID:
    send: string | number
    receive: string`}
        </Highlight>
        <p>
          With this setting, the <code>ID</code> scalar type will be mapped to
          the <code>string | number</code> type in the send situations, and to
          the <code>string</code> type in the receive situations.
        </p>
        <p>
          This setting reflects the fact that an <code>ID</code> value is always
          serialized as a string, but the server can also accept an integer and
          coerce it to a string.
        </p>
        <p>
          The coercion takes place when you <em>receive</em> a value from the
          network; namely, when you receive an operation output and when you
          receive a resolver input. Hence the <i>receive</i> type used for these
          situations.
        </p>
        <p>
          In contrast, when you <em>send</em> a value to the network, you can
          specify a value before coercion. This applies to when you generate a
          resolver output (that will be sent through the network) and when you
          specify an operation input.
        </p>

        <h4 id="separate-form">Separate Form</h4>
        <p>
          The separate form allows you to specify four different types
          separately. This form is not used by built-in scalars.
        </p>
        <p>
          You can use this form if you want to have full control on what type is
          used in each situation.
        </p>
        <p>
          This form is supported for completeness, but we are not aware of any
          use case. If you know one, please get in touch!
        </p>
        <p>
          <b>Example:</b>
        </p>
        <Highlight language="yaml">{`scalarTypes:
  Date:
    resolverInput: string
    resolverOutput: Date | string
    operationInput: string
    operationOutput: string
`}</Highlight>

        <h3 id="notes-on-graphql-code-generator-compatibility">
          Notes on GraphQL Code Generator Compatibility
        </h3>
        <p>
          If you know or trying to migrate from GraphQL Code Generator, you may
          wonder about compatibility with GraphQL Code Generator&apos;s{" "}
          <a
            href="https://the-guild.dev/graphql/codegen/plugins/typescript/typescript#scalars"
            target="_blank"
          >
            <code>scalars</code>
          </a>{" "}
          option.
        </p>
        <p>
          Particularly, GraphQL Code Generator supports specifying a
          scalar&apos;s type as a pair of <code>input</code> and{" "}
          <code>output</code> types.
        </p>
        <p>
          You should note that the input/output configuration is{" "}
          <strong>not</strong> equivalent to nitrogql&apos;s send/receive
          configuration mode. The following table shows the correspondence
          between the two:
        </p>
        <table>
          <thead>
            <tr>
              <th>Name</th>
              <th>input/output</th>
              <th>send/receive</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td>resolverInput</td>
              <td>input</td>
              <td>receive</td>
            </tr>
            <tr>
              <td>resolverOutput</td>
              <td>output</td>
              <td>send</td>
            </tr>
            <tr>
              <td>operationInput</td>
              <td>input</td>
              <td>send</td>
            </tr>
            <tr>
              <td>operationOutput</td>
              <td>output</td>
              <td>receive</td>
            </tr>
          </tbody>
        </table>
        <p>
          We have chosen to diverge from GraphQL Code Generator here because we
          found that the send/receive model better reflects the reality than the
          input/output model.
        </p>
        <p>
          When you are migrating from GraphQL Code Generator, we recommend to
          translate the input/output setting to the send/receive setting.
        </p>
        <p>
          If you need to keep the behavior as-is, you can use the separate form
          instead.
        </p>
      </main>
    </Toc>
  );
}
