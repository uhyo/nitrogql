import Link from "next/link";
import { Hint } from "@/app/_utils/Hint";
import { Highlight } from "@/app/_utils/Highlight";
import { Toc } from "../../_toc";
import { Breadcrumb } from "@/app/_utils/Breadcrumb";

export const metadata = {
  title: "Getting Started",
};

export default function GettingStarted() {
  return (
    <Toc>
      <main>
        <Breadcrumb
          parents={[{ label: "Guides", href: "/guides" }]}
          current="Getting Started"
        />
        <h2>Getting Started</h2>
        <p>
          This page guides you through the steps to get started with nitrogql.
        </p>

        <h3 id="installation">Installation</h3>
        <p>
          The nitrogql CLI can be installed with npm. The CLI is required for
          any workflow using nitrogql. Since generated types depend on{" "}
          <a
            href="https://github.com/dotansimha/graphql-typed-document-node"
            target="_blank"
          >
            TypedDocumentNode
          </a>
          , you also need to install it to your project.
        </p>
        <Highlight language="bash">{`npm install --save-dev @nitrogql/cli @graphql-typed-document-node/core`}</Highlight>
        <p>
          Depending on framework or tool you use, you may also need to install
          appropriate loader packages.
        </p>
        <p>
          For <b>webpack-based tools</b>:
        </p>
        <Highlight language="bash">{`npm install --save-dev @nitrogql/graphql-loader`}</Highlight>
        <p>
          For <b>Rollup-based tools</b>:
        </p>
        <Highlight language="bash">{`npm install --save-dev @nitrogql/rollup-plugin`}</Highlight>

        <h3 id="configuration-file">Configuration File</h3>
        <p>
          nitrogql uses a configuration file to specify the location of your
          schema and operations. The configuration file is named{" "}
          <code>graphql.config.yaml</code>. The configuration file should be
          placed in the root of your project.
        </p>
        <p>The simplest configuration file will look like:</p>
        <Highlight language="yaml">
          {`schema: ./schema/*.graphql
documents: ./app/**/*.graphql`}
        </Highlight>
        <p>
          This file follows{" "}
          <a href="https://the-guild.dev/graphql/config/docs" target="_blank">
            the GraphQL Config convention
          </a>{" "}
          from The Guild. You might already have a configuration file if you use
          other GraphQL tools.
        </p>
        <p>
          See <Link href="/configuration">Configuration</Link> for full details
          about the configuration file. You can also use JSON (
          <code>graphql.config.json</code>) or JavaScript (
          <code>graphql.config.js</code>) instead of YAML.
        </p>

        <h3 id="setting-up-graphql-loader-for-webpack">
          Setting up GraphQL loader for webpack
        </h3>
        <p>
          If you are using webpack-based tools, you need to configure the
          loader.
        </p>
        <p>Example setup (webpack.config.js):</p>
        <Highlight language="javascript">
          {`module: {
  rules: [
    // ...
    {
      test: /\\.graphql$/,
      loader: "@nitrogql/graphql-loader",
      options: {
        configFile: "./graphql.config.yaml",
      }
    }
  ]
}`}
        </Highlight>
        <Hint>
          🔥 We have a Next.js example:{" "}
          <a
            href="https://github.com/uhyo/nitrogql/tree/master/examples/nextjs"
            target="_blank"
          >
            see on GitHub
          </a>
        </Hint>

        <h3 id="setting-up-graphql-loader-for-rollup">
          Setting up GraphQL loader for Rollup
        </h3>
        <p>
          If you are using Rollup-based tools, you need to configure the plugin.
        </p>
        <p>Example setup (rollup.config.js):</p>
        <Highlight language="javascript">
          {`import nitrogql from "@nitrogql/rollup-plugin";

export default {
  plugins: [
    nitrogql({
      include: ["**/*.graphql"],
    }),
  ],
};
`}
        </Highlight>
        <Hint>
          🔥 We have a Vite example:{" "}
          <a
            href="https://github.com/uhyo/nitrogql/tree/master/examples/vite"
            target="_blank"
          >
            see on GitHub
          </a>
        </Hint>

        <hr />
        <Hint>
          🧺 <b>Read Next</b>:{" "}
          <Link href="/guides/using-graphql">
            Using GraphQL in TypeScript projects
          </Link>
        </Hint>
      </main>
    </Toc>
  );
}
