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
          Currently, one built-in plugin is available. Third-party plugins are
          not supported yet.
        </p>
        <ul>
          <li>
            <Link href="/references/plugin-model">nitrogql:model plugin</Link>
          </li>
        </ul>
      </main>
    </Toc>
  );
}
