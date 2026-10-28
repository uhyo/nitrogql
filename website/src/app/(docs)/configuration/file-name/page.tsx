import { Highlight } from "@/app/_utils/Highlight";
import { Toc } from "../../_toc";
import { Breadcrumb } from "@/app/_utils/Breadcrumb";
import { ogp } from "@/app/_utils/metadata";

export const metadata = ogp({
  title: "Configuration File Name",
});

export default function ConfigurationFileName() {
  return (
    <Toc>
      <main>
        <Breadcrumb
          parents={[{ label: "Configuration", href: "/configuration" }]}
          current="Configuration File Name"
        />
        <h2>Configuration File Name</h2>
        <p>
          A configuration file is to be placed in the root of your project. The
          nitrogql CLI will automatically search for a configuration file in the
          current directory.
        </p>

        <h3 id="config-file-name">Default config file name</h3>
        <p>
          By default, configuration file is searched in the following order:
        </p>
        <ol>
          <li>
            <code>graphql.config.json</code>
          </li>
          <li>
            <code>graphql.config.yaml</code>
          </li>
          <li>
            <code>graphql.config.yml</code>
          </li>
          <li>
            <code>graphql.config.js</code>
          </li>
          <li>
            <code>graphql.config.mjs</code>
          </li>
          <li>
            <code>graphql.config.cjs</code>
          </li>
          <li>
            <code>graphql.config.ts</code>
          </li>
          <li>
            <code>graphql.config.mts</code>
          </li>
          <li>
            <code>graphql.config.cts</code>
          </li>
          <li>
            <code>.graphqlrc</code>
          </li>
          <li>
            <code>.graphqlrc.json</code>
          </li>
          <li>
            <code>.graphqlrc.yaml</code>
          </li>
          <li>
            <code>.graphqlrc.yml</code>
          </li>
          <li>
            <code>.graphqlrc.js</code>
          </li>
          <li>
            <code>.graphqlrc.mjs</code>
          </li>
          <li>
            <code>.graphqlrc.cjs</code>
          </li>
          <li>
            <code>.graphqlrc.ts</code>
          </li>
          <li>
            <code>.graphqlrc.mts</code>
          </li>
          <li>
            <code>.graphqlrc.cts</code>
          </li>
        </ol>

        <h3 id="using-javascript-configuration-files">
          Using JavaScript/TypeScript configuration files
        </h3>
        <p>
          nitrogql supports configuration files written in JavaScript or
          TypeScript (both CommonJS and ES Module syntax are supported). When
          using a JavaScript configuration file, an ES Module configuration file
          should default-export the configuration object. A CommonJS
          configuration file should assign the configuration object to{" "}
          <code>module.exports</code>.
        </p>
        <Highlight language="ts">
          {`import type { NitrogqlConfig } from "@nitrogql/cli";

const config: NitrogqlConfig = {
  schema: "./schema/*.graphql",
  documents: ["./app/**/*.graphql", "./common/**/*.graphql"],
  // ...
};

export default config;`}
        </Highlight>
        <p>
          TypeScript files (configuration files and schema files) are run with
          Node.js&apos;{" "}
          <a
            href="https://nodejs.org/docs/latest/api/typescript.html#type-stripping"
            target="_blank"
          >
            type stripping
          </a>{" "}
          when it is available. Therefore, they need to follow its rules: for
          example, imports that are only used as types need the{" "}
          <code>type</code> keyword, and imports of other TypeScript files need
          the <code>.ts</code> extension.
        </p>
        <p>
          To run them with esbuild instead, set the{" "}
          <code>NITROGQL_DISABLE_TYPE_STRIPPING</code> environment variable to{" "}
          <code>1</code>. Then, for example, <code>import</code>s without the{" "}
          <code>type</code> keyword and imports of <code>.ts</code> files with
          the <code>.js</code> extension are also supported.
        </p>
      </main>
    </Toc>
  );
}
