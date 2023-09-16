import Link from "next/link";
import { Highlight } from "@/app/_utils/Highlight";
import { Toc } from "../../_toc";
import { Breadcrumb } from "@/app/_utils/Breadcrumb";
import { ogp } from "@/app/_utils/metadata";

export const metadata = ogp({
  title: "Operation file reference",
});

export default function OperationFile() {
  return (
    <Toc>
      <main>
        <Breadcrumb
          parents={[{ label: "References", href: "/references" }]}
          current="Operation file reference"
        />
        <h2>Operation file reference</h2>
        <p>
          The <b>operation type definition file</b> is generated next to each
          GraphQL operation file. It expresses the types of the GraphQL
          operation, especially the types of variables and the type of the
          response.
        </p>
        <p>
          With the default setting, the file has a <code>.d.graphql.ts</code>{" "}
          extension. This is to tell TypeScript what happens if you import the
          corresponding <code>.graphql</code> file.
        </p>
        <p>
          This page documents what you can import from the <code>.graphql</code>{" "}
          file.
        </p>

        <h3>Operation Object</h3>
        <p>
          The operation object is the type of the object you can pass to a
          GraphQL client to execute the operation. It has the type{" "}
          <code>TypedDocumentNode&lt;Result, Variables&gt;</code>, where{" "}
          <code>Result</code> is the type of the response and{" "}
          <code>Variables</code> is the type of the variables required by the
          operation.
        </p>
        <p>
          With the default setting, the operation object is exported as a
          default export. You can import it like:
        </p>
        <Highlight language="typescript">
          {`import getTodosQuery from "./getTodos.graphql";`}
        </Highlight>
        <p>
          By configuring the{" "}
          <Link href="/configuration/options#generate.export.defaultExportForOperation">
            <code>defaultExportForOperation</code>
          </Link>{" "}
          option, you can change this to a named export:
        </p>
        <Highlight language="typescript">
          {`import { getTodosQuery } from "./operation.graphql";`}
        </Highlight>
        <p>
          The name of the export is determined by the name of the operation. See
          the references for{" "}
          <Link href="/configuration/options#generate.name">
            <code>generate.name</code>
          </Link>{" "}
          on how to configure generated names.
        </p>

        <h3 id="results-and-variables">Results and Variables</h3>
        <p>
          By default, the operation object is the only thing exported from the
          file. However, you can configure the{" "}
          <Link href="/configuration/options#generate.export">
            <code>generate.export</code>
          </Link>{" "}
          option to export other things as well.
        </p>
        <p>
          If you configure the option to export results and variables, they are
          exported as named exports.
        </p>
      </main>
    </Toc>
  );
}
