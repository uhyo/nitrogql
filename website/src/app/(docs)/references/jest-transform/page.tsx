import Link from "next/link";
import { Highlight } from "@/app/_utils/Highlight";
import { Toc } from "../../_toc";
import { Breadcrumb } from "@/app/_utils/Breadcrumb";
import { ogp } from "@/app/_utils/metadata";
import { Hint } from "@/app/_utils/Hint";

export const metadata = ogp({
  title: "@nitrogql/jest-transform reference",
});

export default function JestTransform() {
  return (
    <Toc>
      <main>
        <Breadcrumb
          parents={[{ label: "References", href: "/references" }]}
          current="@nitrogql/jest-transform reference"
        />
        <h2>
          <code>@nitrogql/jest-transform</code> reference
        </h2>
        <p>
          <code>@nitrogql/jest-transform</code> is a Jest transformer that lets
          you load <code>.graphql</code> files from your code during testing.
          This package is suited for projects that use Jest to test client-side
          code that uses GraphQL.
        </p>
        <h3 id="example">Example</h3>
        <p>
          <b>jest.config.js</b>
        </p>
        <Highlight language="ts">{`{
  "transform": {
    "^.+\\.graphql$": ["@nitrogql/jest-transform", {
      "configFile":  path.resolve(__dirname, "nitrogql.config.js")
    }]
  }
}`}</Highlight>

        <h3 id="options">options</h3>

        <h4 id="configfile">options.configFile</h4>
        <p>
          Path to the{" "}
          <Link href="/configuration/file-name">configuration file</Link>.
        </p>

        <h4 id="additionalTransformer">options.additionalTransformer</h4>
        <p>
          Additional transformer to apply to the generated source code. Can be a
          string or an array of two elements: the transformer name and the
          transformer configuration. See below for more information.
        </p>

        <h4 id="additionalTransformerFilenameSuffix">
          options.additionalTransformerFilenameSuffix
        </h4>
        <p>
          Suffix to add to filename when passing code to the additional
          transformer. Defaults to <code>&quot;.js&quot;</code>.
        </p>

        <h3 id="commonjs-support">CommonJS Support</h3>
        <p>
          <code>@nitrogql/jest-transform</code> is only capable of transforming
          GraphQL files to ES modules. If you need CommonJS support, use the{" "}
          <code>additionalTransformer</code> option to apply another transformer
          that can convert ES modules to CommonJS.
        </p>
        <p>
          Example setup with <code>ts-jest</code>:
        </p>
        <Highlight language="js">{`{
  "transform": {
    "^.+\.tsx?": ["ts-jest", { isolatedModules: true }],
    "^.+\\.graphql$": ["@nitrogql/jest-transform", {
      "configFile":  path.resolve(__dirname, "nitrogql.config.yml"),

      // Use the additionalTransformer option to apply ts-jest to the output.
      "additionalTransformer": ["ts-jest", { isolatedModules: true }],

      // ts-jest expects .ts files, so we need to change the file extension
      // by applying the additionalTransformerFilenameSuffix option.
      "additionalTransformerFilenameSuffix": ".ts"
    }]
  },
}`}</Highlight>
      </main>
    </Toc>
  );
}
