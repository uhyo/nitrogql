import { Hint } from "@/app/(utils)/Hint";
import { Highlight } from "@/app/(utils)/Highlight";

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

      <h4>
        <code>--output-format</code>
      </h4>
      <p>Specify the output format of the CLI. Possible values are:</p>
      <ul>
        <li>
          <code>human</code> (default): prints human-readable output to stderr.
        </li>
        <li>
          <code>rdjson</code>: prints &apos;check&apos; results in{" "}
          <a
            href="https://github.com/reviewdog/reviewdog/tree/master/proto/rdf"
            target="_blank"
          >
            rdjson
          </a>{" "}
          format to stdout. This is useful for integrating with reviewdog. Makes
          sense only when the <code>check</code> command is run.
        </li>
        <li>
          <code>json</code>: prints nitrogql specific JSON output to stdout.
        </li>
      </ul>
      <p>
        The signature of the <code>json</code> output is:
      </p>
      <Highlight language="typescript">{`interface CLIOutput {
  /**
   * Exists when a command fails.
   */
  error?: {
    /**
     * Command name that had error.
     */
    command: string | null;
    /**
     * Error message.
     */
    message: string;
  }
  /**
   * Exists when the 'check' command is run.
   */
  check?: {
    /**
     * List of errors.
     * Empty when the check is successful.
     */
    errors: {
      fileType: "schema" | "operation";
      file?: {
        path: string;
        // line and column are 0-indexed
        line: number;
        column: number;
      }
      message: string;
    }[]
  }
  /**
   * Exists when the 'generate' command is run.
   */
  generate?: {
    /**
     * List of output files.
     */
    files: { 
      fileType:
        | "schemaTypeDefinition"
        | "schemaTypeDefinitionSourceMap"
        | "operationTypeDefinition"
        | "operationTypeDefinitionSourceMap";
      path: string;
    }[];
  }
}`}</Highlight>

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
