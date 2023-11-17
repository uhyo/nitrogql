import Link from "next/link";
import { Highlight } from "@/app/_utils/Highlight";
import { Toc } from "../../_toc";
import { Breadcrumb } from "@/app/_utils/Breadcrumb";
import { ogp } from "@/app/_utils/metadata";

export const metadata = ogp({
  title: "@nitrogql/cli reference",
});

export default function OperationFile() {
  return (
    <Toc>
      <main>
        <Breadcrumb
          parents={[{ label: "References", href: "/references" }]}
          current="@nitrogql/cli reference"
        />
        <h2>
          <code>@nitrogql/cli</code> reference
        </h2>
        <p>
          This page describes usage of <code>@nitrogql/cli</code> as a library.
          For usage as a CLI, see <Link href="/cli">CLI Usage</Link>.
        </p>
        <p>
          <code>@nitrogql/cli</code> exports types that help writing config
          files.
        </p>
        <h3 id="nitrogqlconfig">
          <code>NitrogqlConfig</code>
        </h3>
        <p>
          Type of the entire object exported by{" "}
          <Link href="/configuration/options">
            <code>graphql.config.ts</code>
          </Link>
          .
        </p>
        <p>
          <b>Example:</b>
        </p>
        <Highlight language="ts">{`import type { NitrogqlConfig } from "@nitrogql/cli";

const config: NitrogqlConfig = {
  schema: "src/schema/*.graphql",
  documents: "src/app/**/*.graphql",
  extensions: {
    nitrogql: {
      plugins: ["nitrogql:model-plugin"],
      // ...
    }
  }
};

export default config;`}</Highlight>

        <h3 id="nitrogqlextension">NitrogqlExtension</h3>
        <p>
          Type of <code>extensions.nitrogql</code> in the config file. Useful
          when you want to mix nitrogql with other tools.
        </p>
        <p>
          <b>Example:</b>
        </p>
        <Highlight language="ts">{`import type { NitrogqlExtension } from "@nitrogql/cli";

const nitrogql: NitrogqlExtension = {
  plugins: ["nitrogql:model-plugin"],
  // ...
};

const config = {
  schema: "src/schema/*.graphql",
  documents: "src/app/**/*.graphql",
  extensions: {
    nitrogql,
    // ...
  }
};

export default config;`}</Highlight>
      </main>
    </Toc>
  );
}
