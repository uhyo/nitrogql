import Link from "next/link";
import { Highlight } from "@/app/_utils/Highlight";
import { Toc } from "../../_toc";
import { Breadcrumb } from "@/app/_utils/Breadcrumb";
import { ogp } from "@/app/_utils/metadata";
import { Hint } from "@/app/_utils/Hint";

export const metadata = ogp({
  title: "@nitrogql/rollup-plugin reference",
});

export default function RollupPlugin() {
  return (
    <Toc>
      <main>
        <Breadcrumb
          parents={[{ label: "References", href: "/references" }]}
          current="@nitrogql/rollup-plugin reference"
        />
        <h2>
          <code>@nitrogql/rollup-plugin</code> reference
        </h2>
        <p>
          <code>@nitrogql/rollup-plugin</code> is a Rollup plugin that processes
          GraphQL files. This package is suited for projects that use Rollup or
          Vite as a build tool.
        </p>
        <h3 id="example">Example</h3>
        <p>
          <b>rollup.config.js</b>
        </p>
        <Highlight language="ts">{`import nitrogql from "@nitrogql/rollup-plugin";
{
  plugins: [
    nitrogql({
      configFile: "./graphql.config.ts",
      include: ["**/*.graphql"],
    }),
  ],
};`}</Highlight>

        <h3 id="options">options</h3>

        <h4 id="configfile">options.configFile</h4>
        <p>
          Path to the{" "}
          <Link href="/configuration/file-name">configuration file</Link>.
          Relative paths are resolved relative to the project root in case of
          Vite, and relative to the current working directory in case of Rollup.
        </p>
        <Hint>
          🚟 When omitted, no configuration file is loaded, meaning that the
          default configuration is used. We recommend always specifying a
          configuration file.
        </Hint>

        <h4 id="include">options.include</h4>
        <p>An array of glob patterns that specify the files to process.</p>

        <h4 id="exclude">options.exclude</h4>
        <p>
          An array of glob patterns that specify the files to ignore even if
          they match the include patterns.
        </p>
      </main>
    </Toc>
  );
}
