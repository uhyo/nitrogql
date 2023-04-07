import Image from "next/image";
import Link from "next/link";
import { Hint } from "@/app/(utils)/Hint";
import { Highlight } from "@/app/(utils)/Highlight";
import { Figures } from "@/app/(utils)/Figures";

export default function GettingStarted() {
  return (
    <main>
      <h2>nitrogql CLI Usage</h2>
      <p>
        nitrogql provides a CLI to check your GraphQL code and to generate types
        from your schema and operations.
      </p>
      <p>
        The CLI is installed with <code>npm</code>:
      </p>
      <Highlight language="bash">{`npm install --save-dev @nitrogql/cli`}</Highlight>

      <h3>commands</h3>
      <p>Basic usage of the CLI is:</p>
      <Highlight language="bash">{`nitrogql <command> [options]`}</Highlight>
      <p>The following commands are available.</p>
      <ul>
        <li>
          <code>check</code>: Check your GraphQL code.
        </li>
        <li>
          <code>generate</code>: Generate types from your schema and operations.
        </li>
      </ul>
      <Hint>
        💡 <code>generate</code> also implies <code>check</code>. GraphQL code
        is checked before types are generated.
      </Hint>

      <h3>options</h3>

      <h4>
        <code>--config-file</code>
      </h4>
      <p>
        Specify the path to the configuration file. By default, the CLI looks
        for <code>graphql.config.yaml</code> in the current directory.
      </p>
      <Highlight language="bash">{`npx nitrogql generate --config-file ./path/to/config.yaml`}</Highlight>

      <h4>
        <code>--schema</code>
      </h4>
      <p>
        Specify the path to the schema file(s). This overrides the schema path
        specified in the configuration file.
      </p>

      <h4>
        <code>--operation</code>
      </h4>
      <p>
        Specify the path to the operation file(s). This overrides the operation
        path specified in the configuration file.
      </p>

      <h3>Notes on file system access</h3>
      <p>
        Due to the security nature of WASI, the CLI cannot access files outside
        the current directory by default. To allow access to files outside the
        current directory, use a <code>NITROGQL_FS_SCOPE</code> environment
        variable.
      </p>
      <Highlight language="bash">{`NITROGQL_FS_SCOPE=../ npx nitrogql check --config-file ../graphql.config.yaml`}</Highlight>
    </main>
  );
}
