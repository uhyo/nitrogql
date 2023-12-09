import { Highlight } from "@/app/_utils/Highlight";
import { ArticleMetadata } from "./_meta";
import Link from "next/link";

export const blogPostRelease1_4: ArticleMetadata = {
  slug: "release-1.4",
  title:
    "nitrogql 1.4 release: Maybe the first step towards Fragment Colocation?",
  shortDescription: `In nitrogql 1.4, we added support for importing fragments from other GraphQL documents.
Previously you had to define all fragments in the same document as the operation that uses them.
The ability to import fragments from other documents is a step towards enabling Fragment Colocation.`,
  publishDate: new Date("2023-12-09T00:00Z"),
  render,
};

function render() {
  return (
    <>
      <p>
        Today, we are happy to announce release of <strong>nitrogql 1.4</strong>
        !
      </p>
      <p>
        <b>nitrogql</b> is a toolchain for using GraphQL in TypeScript projects.
        In 1.4, we added support for importing fragments from other GraphQL
        documents. Previously you had to define all fragments in the same
        document as the operation that uses them.
      </p>

      <h3 id="the-import-syntax">
        The <code>#import</code> syntax
      </h3>
      <p>
        nitrogql 1.4 introduces a new syntax for importing fragments from other
        GraphQL documents. It looks like this:
      </p>
      <Highlight language="graphql">{`#import Fragment1, Fragment2 from "./path/to/file.graphql"`}</Highlight>
      <p>
        Above is an example of importing two fragments from{" "}
        <code>path/to/file.graphql</code> (relative to the current file). The
        fragments are then available for use in the current document. Of course,
        the imported document must define these fragments.
      </p>
      <p>
        You can also import all fragments from a document by using the{" "}
        <code>*</code> wildcard:
      </p>
      <Highlight language="graphql">{`#import * from "./path/to/file.graphql"`}</Highlight>
      <p>
        This will import all fragments from <code>path/to/file.graphql</code>.
      </p>
      <p>
        While the new syntax isn&apos;t part of the GraphQL specification, it is
        inspired by{" "}
        <a
          href="https://the-guild.dev/graphql/tools/docs/schema-loading"
          target="_blank"
        >
          The Guild&apos;s similar syntax
        </a>
        . Some of you may already be familiar with it.
      </p>

      <h3 id="implications-of-the-new-syntax">
        Implications of the new syntax
      </h3>
      <p>
        Placing fragments in separate files is often done in the context of{" "}
        <b>Fragment Colocation</b>. Fragment Colocation is a practice where
        fragments are placed near (often in the same directory as) the component
        that uses them. One GraphQL query is prepared per page and it collects
        all fragments that are necessary for the component to render.
      </p>
      <p>
        The addition of the <code>#import</code> syntax enables this use case
        for nitrogql users.
      </p>
      <p>
        However, this is only the first step towards Fragment Colocation.
        Literature on Fragment Colocation often mentions that fragments be given
        an opaque type in operation results and that users can obtain actual
        contents by using a special function (e.g. <code>useFragment</code>).
      </p>
      <p>
        The current version of nitrogql does not support this use case. We may
        add support for it in the future if there is demand for it.
      </p>
      <p>
        For now, we hope that the new <code>#import</code> syntax will make your
        GraphQL development experience a bit better. Working examples of how to
        use the syntax are available in the{" "}
        <a
          href="https://github.com/uhyo/nitrogql/tree/master/examples"
          target="_blank"
        >
          examples directory
        </a>
        .
      </p>

      <h3 id="conclusion">Conclusion</h3>
      <p>
        In this release, we added support for importing fragments from other
        GraphQL documents. This is a step towards enabling Fragment Colocation.
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
