import Link from "next/link";
import { Highlight } from "@/app/_utils/Highlight";
import { Toc } from "../../_toc";
import { Breadcrumb } from "@/app/_utils/Breadcrumb";
import { ogp } from "@/app/_utils/metadata";
import { Hint } from "@/app/_utils/Hint";

export const metadata = ogp({
  title: "@nitrogql/graphql-loader reference",
});

export default function GraphQLLoader() {
  return (
    <Toc>
      <main>
        <Breadcrumb
          parents={[{ label: "References", href: "/references" }]}
          current="@nitrogql/graphql-loader reference"
        />
        <h2>
          <code>@nitrogql/graphql-loader</code> reference
        </h2>
        <p>
          <code>@nitrogql/graphql-loader</code> is a Webpack loader that
          processes GraphQL files. This package is suited for projects that use
          Webpack or Next.js as a build tool.
        </p>
        <h3 id="example">Example</h3>
        <p>
          <b>webpack.config.js</b>
        </p>
        <Highlight language="ts">{`module.exports = {
  module: {
    rules: [
      {
        test: /\.graphql$/,
        loader: "@nitrogql/graphql-loader",
        options: {
          configFile: "./graphql.config.yaml",
        },
      },
    ],
  },
};`}</Highlight>

        <h3 id="options">options</h3>
        <p>
          Currently, <code>@nitrogql/graphql-loader</code> supports only one
          option: <code>configFile</code>.
        </p>

        <h4 id="configfile">options.configFile</h4>
        <p>
          Path to the{" "}
          <Link href="/configuration/file-name">configuration file</Link>.
          Relative paths are resolved relative to{" "}
          <a
            href="https://webpack.js.org/configuration/entry-context/#context"
            target="_blank"
          >
            Webpack&apos;s context
          </a>
          .
        </p>
        <Hint>
          🚟 When omitted, no configuration file is loaded, meaning that the
          default configuration is used. We recommend always specifying a
          configuration file.
        </Hint>
      </main>
    </Toc>
  );
}
