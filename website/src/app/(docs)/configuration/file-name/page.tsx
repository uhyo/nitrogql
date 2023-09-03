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
        </ol>

        <h3 id="using-javascript-configuration-files">
          Using JavaScript configuration files
        </h3>
        <p>
          nitrogql supports configuration files written in JavaScript (both
          CommonJS and ES Module syntax are supported). When using a JavaScript
          configuration file, an ES Module configuration file should
          default-export the configuration object. A CommonJS configuration file
          should assign the configuration object to <code>module.exports</code>.
        </p>
        <p>Also you can use JSDoc for type checking and auto-completion.</p>
        <Highlight language="js">
          {`/**
  * @type {import("@nitrogql/cli").NitrogqlConfig}
  */
const config = {
  schema: "./schema/*.graphql",
  documents: ["./app/**/*.graphql", "./common/**/*.graphql"],
  // ...
};

export default config;`}
        </Highlight>
      </main>
    </Toc>
  );
}
