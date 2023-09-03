import Link from "next/link";
import { Toc } from "../_toc";
import { InPageNav } from "@/app/_utils/InPageNav";

export const metadata = {
  title: "Guides",
};

export default function Guides() {
  return (
    <Toc>
      <main>
        <h2>Guides</h2>
        <InPageNav>
          <Link href="/guides/getting-started">Getting Started</Link>
          <Link href="/guides/using-graphql">
            Using GraphQL in TypeScript projects
          </Link>
          <Link href="/guides/monorepo">Monorepo Guide</Link>
          <Link href="/guides/migrating-from-graphql-codegen">
            Migrating from GraphQL Code Generator
          </Link>
        </InPageNav>
      </main>
    </Toc>
  );
}
