import Link from "next/link";
import { Highlight } from "@/app/_utils/Highlight";
import { ArticleMetadata } from "./_meta";

export const blogPostRelease2_1: ArticleMetadata = {
  slug: "release-2.1",
  title: "nitrogql 2.1 release: TypeScript content mapper, stricter checks",
  shortDescription: `nitrogql 2.1 ships an experimental content mapper for TypeScript 7.1,
which lets TypeScript read your .graphql files directly without generated files.
This release also brings stricter static checks and stable error codes.
`,
  publishDate: new Date("2026-11-06T00:00Z"),
  render,
};

function render() {
  return (
    <>
      <p>
        Today, we are happy to announce release of <strong>nitrogql 2.1</strong>
        !
      </p>
      <p>
        <b>nitrogql</b> is a toolchain for using GraphQL in TypeScript projects.
        The highlight of 2.1 is an experimental <strong>content mapper</strong>{" "}
        for TypeScript 7.1, which gives you types for your GraphQL files without
        running <code>nitrogql generate</code>. This release also makes{" "}
        <code>nitrogql check</code> stricter, adds stable error codes to
        diagnostics, and changes how configuration files are loaded.
      </p>

      <h2 id="content-mapper">
        Experimental: TypeScript content mapper for GraphQL files
      </h2>
      <p>
        TypeScript 7.1 introduced <strong>content mappers</strong>, a plugin
        mechanism that lets TypeScript understand files other than TypeScript
        and JavaScript. nitrogql 2.1 provides a content mapper for{" "}
        <code>.graphql</code> files.
      </p>
      <p>
        Until now, <code>nitrogql generate</code> generated a{" "}
        <code>.d.graphql.ts</code> file next to each <code>.graphql</code> file,
        and TypeScript read the generated file when you imported a{" "}
        <code>.graphql</code> file. With the content mapper, TypeScript asks
        nitrogql for the types of a <code>.graphql</code> file whenever it needs
        them. Nothing is written to the disk, so there are no generated files to
        keep up to date, and your editor reflects your edits to GraphQL files
        immediately.
      </p>
      <p>
        To use it, register nitrogql as a content mapper in your{" "}
        <code>tsconfig.json</code>:
      </p>
      <Highlight language="json">
        {`{
  "compilerOptions": {
    // ...
  },
  "contentMappers": [
    {
      "package": "@nitrogql/cli",
      "extensions": [".graphql"]
    }
  ]
}`}
      </Highlight>
      <p>
        Your nitrogql configuration file is used as usual. Since content mappers
        are external programs executed by TypeScript, you need to explicitly
        allow running them, e.g. with <code>tsc --runExternalCode</code>.
      </p>
      <p>
        The content mapper is <strong>experimental</strong>, because content
        mappers themselves are a new feature of TypeScript 7.1, which is still
        in beta. Its behavior and options may change in minor releases of
        nitrogql. <code>nitrogql generate</code> remains the stable way to use
        nitrogql. See{" "}
        <Link href="/guides/typescript-content-mapper">
          Using TypeScript Content Mapper
        </Link>{" "}
        for setup instructions and details. Feedback is very welcome!
      </p>

      <h2 id="stricter-checks">Stricter static checks</h2>
      <p>
        <code>nitrogql check</code> got stricter in this release. These changes
        may report errors in documents that passed before, so please check your
        project when upgrading.
      </p>
      <h3 id="field-selection-merging">Field Selection Merging</h3>
      <p>
        The checker now implements the{" "}
        <a
          href="https://spec.graphql.org/October2021/#sec-Field-Selection-Merging"
          target="_blank"
        >
          Field Selection Merging
        </a>{" "}
        validation rule of the GraphQL specification. Selections that share a
        response name must be mergeable: they must have the same shape, and,
        when they can apply to the same object, select the same field with the
        same arguments.
      </p>
      <Highlight language="graphql">
        {`query {
  me {
    name
    # Error: conflicts with 'name' above
    name: friends {
      id
    }
  }
}`}
      </Highlight>
      <p>
        Spec-compliant GraphQL servers reject such documents anyway, so this
        check catches at build time what would otherwise fail at runtime.
      </p>
      <h3 id="no-transitive-imports">No implicit transitive imports</h3>
      <p>
        A file can now only spread fragments that it defines or imports by
        itself. Previously, a fragment imported by an imported file could be
        used too. If your project relied on this, add the missing{" "}
        <code>#import</code>:
      </p>
      <Highlight language="graphql">
        {`#import UserProfile from "./profile.graphql"
#import UserAvatar from "./avatar.graphql"

query {
  me {
    ...UserProfile
    # UserAvatar is imported by profile.graphql,
    # but this file needs to import it as well
    ...UserAvatar
  }
}`}
      </Highlight>
      <p>
        This makes each file&apos;s dependencies explicit, and matches how the
        content mapper sees your files. See{" "}
        <Link href="/references/syntax-import">the #import reference</Link> for
        details.
      </p>

      <h2 id="error-codes">Stable error codes</h2>
      <p>
        Every kind of GraphQL error now has a stable error code, such as{" "}
        <code>nitrogql4007</code>. The codes are shown in the CLI output and in
        the content mapper&apos;s diagnostics, and are documented on the{" "}
        <Link href="/references/error-codes">Error Codes</Link> page.
      </p>
      <Highlight language="text">
        {`nitrogql4007: Field 'nam' is not found on type 'User'`}
      </Highlight>
      <p>
        In the <code>json</code> output format, each error has a{" "}
        <code>code</code> field, and in the <code>rdjson</code> format, each
        diagnostic includes the code with a link to its documentation. Also,
        syntax errors are now reported as structured diagnostics in these
        formats, just like other errors.
      </p>

      <h2 id="type-stripping">
        Configuration files are loaded with Node.js type stripping
      </h2>
      <p>
        TypeScript configuration files and schema files are now run with
        Node.js&apos;s built-in type stripping by default, instead of
        esbuild-register. On Node.js versions without type stripping, nitrogql
        automatically falls back to esbuild-register.
      </p>
      <p>
        With type stripping, your TypeScript files must follow Node.js&apos;s
        rules. Most notably, imports used only as types need the{" "}
        <code>type</code> keyword:
      </p>
      <Highlight language="typescript">
        {`// OK
import type { NitrogqlConfig } from "@nitrogql/cli";
// Error: does not provide an export named 'NitrogqlConfig'
import { NitrogqlConfig } from "@nitrogql/cli";`}
      </Highlight>
      <p>
        Also, other <code>.ts</code> files must be imported with the{" "}
        <code>.ts</code> extension, and enums and namespaces are not supported.
        If you need the old behavior, set the{" "}
        <code>NITROGQL_DISABLE_TYPE_STRIPPING</code> environment variable to{" "}
        <code>1</code>. See{" "}
        <Link href="/configuration/file-name">Configuration File Name</Link> for
        details.
      </p>

      <h2 id="generated-types">Improvements to generated types</h2>
      <ul>
        <li>
          In generated resolver types, nullable arguments without a default
          value are now optional, matching graphql-js, which leaves them out of{" "}
          <code>args</code> when the client omits them.
        </li>
        <li>
          Operation types now merge every compatible combination of nested
          selections that use different <code>@skip</code>/<code>@include</code>{" "}
          variables. Previously, some combinations were missing from the
          generated type.
        </li>
        <li>
          Fields omitted by <code>@skip</code>/<code>@include</code> are now
          typed as optional <code>never</code> instead of a required{" "}
          <code>undefined</code>.
        </li>
        <li>
          Selections are now printed as plain object types, which makes type
          checking of operation types noticeably faster.
        </li>
      </ul>

      <h2 id="bug-fixes">Bug fixes</h2>
      <ul>
        <li>
          <code>nitrogql check</code> now keeps validating other documents when
          an import cannot be resolved.
        </li>
        <li>
          The model plugin now uses the field type&apos;s model for field-level{" "}
          <code>@model</code>.
        </li>
        <li>Fixed the native configuration file loader.</li>
        <li>
          Fixed fragments imported from a file that was already reached through
          another import being silently dropped.
        </li>
      </ul>

      <h2 id="conclusion">Conclusion</h2>
      <p>
        nitrogql 2.1 takes a first step towards using GraphQL in TypeScript
        without any generated files, thanks to TypeScript 7.1&apos;s content
        mappers. Please try it out and let us know what you think on{" "}
        <a href="https://github.com/uhyo/nitrogql/issues" target="_blank">
          GitHub Issues
        </a>
        . Along with stricter checks and stable error codes, we hope this
        release makes your GraphQL development even more type-safe.
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
