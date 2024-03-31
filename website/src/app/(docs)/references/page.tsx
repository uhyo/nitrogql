import Link from "next/link";
import { Toc } from "../_toc";

export const metadata = {
  title: "References",
};

export default function References() {
  return (
    <Toc>
      <main>
        <h2>References</h2>
        <h3>Supported Syntax</h3>
        <p>
          nitrogql supports all GraphQL features from the{" "}
          <a href="https://spec.graphql.org/October2021/" target="_blank">
            October 2021 specification
          </a>
          . In addition, it supports the following syntax extension:
        </p>
        <ul>
          <li>
            <Link href="/references/syntax-import">
              <code>#import</code> syntax
            </Link>
          </li>
        </ul>

        <h3>Packages</h3>
        <p>
          Packages not listed here are considered internal. They are not meant
          to be used directly and are subject to change without a major version
          bump.
        </p>

        <ul>
          <li>
            <Link href="/references/nitrogql-cli">@nitrogql/cli</Link>
          </li>
          <li>
            <Link href="/references/graphql-loader">
              @nitrogql/graphql-loader
            </Link>
          </li>
          <li>
            <Link href="/references/rollup-plugin">
              @nitrogql/rollup-plugin
            </Link>
          </li>
          <li>
            <Link href="/references/jest-transform">
              @nitrogql/jest-transform
            </Link>
          </li>
        </ul>
        <h3>Generated Files</h3>
        <p>
          This is a documentation on the details of generated files. Anything
          not covered in it is considered an implementation detail and is
          subject to change without a major version bump.
        </p>
        <ul>
          <li>
            <Link href="/references/schema-file">
              Schema type definition file
            </Link>
          </li>
          <li>
            <Link href="/references/resolvers-file">
              Resolvers type definition file
            </Link>
          </li>
          <li>
            <Link href="/references/operation-file">
              Operation type definition file
            </Link>
          </li>
          <li>
            <Link href="/references/server-graphql-file">
              Server GraphQL schema file
            </Link>
          </li>
        </ul>

        <h3>Plugins</h3>
        <p>
          Currently, two built-in plugins are available. Third-party plugins are
          not supported yet.
        </p>
        <ul>
          <li>
            <Link href="/references/plugin-model">nitrogql:model-plugin</Link>
          </li>
          <li>
            <Link href="/references/plugin-graphql-scalars">
              nitrogql:graphql-scalars-plugin
            </Link>
          </li>
        </ul>
      </main>
    </Toc>
  );
}
