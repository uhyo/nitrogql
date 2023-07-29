import Image from "next/image";
import Link from "next/link";
import { Hint } from "@/app/_utils/Hint";
import { Highlight } from "@/app/_utils/Highlight";
import { Figures } from "@/app/_utils/Figures";

export default function GettingStarted() {
  return (
    <main>
      <h2>nitrogql Recipes</h2>
      <p>
        nitrogql can be used with a variety of frameworks and tools. Find
        examples for common use cases below.
      </p>

      <h3>Next.js</h3>
      <p>
        <a href="https://github.com/uhyo/nitrogql/tree/master/examples/nextjs">
          See example on GitHub
        </a>
      </p>
      <p>
        This example shows how to use nitrogql with <b>Next.js</b>. It
        demonstrates the <b>schema-based</b> approach to GraphQL by containing
        both the server-side and client-side code in a single project.
      </p>
      <p>
        It uses the <code>generate</code> command to generate types from your
        schema and operations. The generated types are used with{" "}
        <b>
          <a href="https://formidable.com/open-source/urql/">urql</a>
        </b>{" "}
        to fetch data from a GraphQL API.
      </p>

      <h3>Vite</h3>
      <p>
        <a href="https://github.com/uhyo/nitrogql/tree/master/examples/vite">
          See example on GitHub
        </a>
      </p>
      <p>
        This example shows how to use nitrogql with <b>Vite</b>. It also
        demonstrates use of nitrogql with <b>external GraphQL API</b>.
        Introspection result of the API is contained in the project and is used
        as a schema definition.
      </p>
      <p>
        nitrogql accepts introspection JSON as a schema definition in place of{" "}
        <code>.graphql</code> files. This allows you to use nitrogql for
        applications that use external GraphQL APIs.
      </p>

      <h3>Want more?</h3>
      <p>
        If you have a use case that is not covered by the examples above, please
        let us know! Also please check out <Link href="/">Top Page</Link> for
        planned features.
      </p>
    </main>
  );
}
