import Link from "next/link";
import { Highlight } from "@/app/_utils/Highlight";
import { Toc } from "../../_toc";
import { Breadcrumb } from "@/app/_utils/Breadcrumb";
import { ogp } from "@/app/_utils/metadata";
import { Hint } from "@/app/_utils/Hint";

export const metadata = ogp({
  title: "The #import syntax",
});

export default function ImportSyntax() {
  return (
    <Toc>
      <main>
        <Breadcrumb
          parents={[{ label: "References", href: "/references" }]}
          current="The #import syntax"
        />
        <h2>
          The <code>#import</code> syntax
        </h2>
        <p>
          The <code>#import</code> syntax is an extension to the GraphQL
          language that allows you to import fragments from other GraphQL
          documents. This syntax is similar to{" "}
          <a
            href="https://the-guild.dev/graphql/tools/docs/schema-loading"
            target="_blank"
          >
            The Guild&apos;s <code>#import</code> extension for schema loading
          </a>
          .
        </p>
        <p>
          In nitrogql, fragments defined in operation documents are scoped
          within the operation document. This means that you cannot reuse
          fragments across operation documents. The <code>#import</code> syntax
          allows you to overcome this limitation.
        </p>
        <Hint>
          💡 The <code>#import</code> syntax is not available in schema
          definition language (SDL) files. This is because unlike operation
          documents, SDL files are not scoped. All SDL files share the same,
          global scope.
        </Hint>

        <h3 id="usage">Usage</h3>
        <p>
          The <code>#import</code> syntax is of the following form:
        </p>
        <Highlight language="graphql">{`#import Fragment1, Fragment2 from "./path/to/file.graphql"`}</Highlight>
        <p>
          Above is an example of importing two fragments from{" "}
          <code>path/to/file.graphql</code> (relative to the current file). The
          fragments are then available for use in the current document. Of
          course, the imported document must define these fragments.
        </p>
        <p>
          You can also import all fragments from a document by using the{" "}
          <code>*</code> wildcard:
        </p>
        <Highlight language="graphql">{`#import * from "./path/to/file.graphql"`}</Highlight>
        <p>
          The above imports all fragments from <code>path/to/file.graphql</code>
          .
        </p>
        <p>
          The <code>#import</code> syntax can be used in the top level of an
          operation document. If the syntax is used elsewhere, it will be a
          syntax error.
        </p>
        <Highlight language="graphql">{`
#import Fragment1 from "./path/to/file.graphql"
# ↑ This is valid

query {
  #import Fragment2 from "./path/to/file.graphql"
  # ↑ This is invalid!
  # ...
}
`}</Highlight>
        <Hint>
          🎞️ Since <code>#</code> is also used for comments, failure to use the{" "}
          <code>#import</code> syntax correctly may result in it being treated
          as a comment instead.
        </Hint>
      </main>
    </Toc>
  );
}
