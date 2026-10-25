import Link from "next/link";
import { Toc } from "../../_toc";
import { Breadcrumb } from "@/app/_utils/Breadcrumb";
import { ogp } from "@/app/_utils/metadata";

export const metadata = ogp({
  title: "Error codes",
});

type ErrorCodeEntry = {
  code: string;
  description: string;
  example: string;
};

const syntaxErrors: readonly ErrorCodeEntry[] = [
  {
    code: "nitrogql1001",
    description: "Syntax error in a GraphQL document.",
    example: "Parse error: expected Selection",
  },
];

const commonErrors: readonly ErrorCodeEntry[] = [
  {
    code: "nitrogql2001",
    description: "Type is not defined.",
    example: "Type 'Usr' is not defined",
  },
  {
    code: "nitrogql2002",
    description: "Directive is not defined.",
    example: "Directive 'unknown' is not defined",
  },
  {
    code: "nitrogql2003",
    description: "Directive is not allowed for this location.",
    example: "Directive 'skip' is not allowed for this location",
  },
  {
    code: "nitrogql2004",
    description: "Non-repeatable directive is applied more than once.",
    example: "Repeated application of directive 'deprecated' is not allowed",
  },
  {
    code: "nitrogql2005",
    description:
      "Arguments are given to something that does not accept arguments.",
    example: "This directive does not accept arguments",
  },
  {
    code: "nitrogql2006",
    description: "Argument is not defined.",
    example: "Argument 'id' is not defined",
  },
  {
    code: "nitrogql2007",
    description: "Required argument is not specified.",
    example: "Required argument 'id' is not specified",
  },
  {
    code: "nitrogql2008",
    description: "Value does not match the expected type.",
    example: "This value does not match expected type 'Int!'",
  },
  {
    code: "nitrogql2009",
    description: "Variable is not defined.",
    example: "Variable '$id' is not defined",
  },
  {
    code: "nitrogql2010",
    description: "Value is not a member of the enum.",
    example: "'PURPLE' is a member of enum Color",
  },
  {
    code: "nitrogql2011",
    description: "Field of an input object is not defined.",
    example: "Field 'nam' is not defined",
  },
  {
    code: "nitrogql2012",
    description: "Required field of an input object is not specified.",
    example: "Field 'name' is required",
  },
  {
    code: "nitrogql2013",
    description:
      "Value of a @oneOf input object does not have exactly one field.",
    example:
      "Value of @oneOf input object 'UserBy' must have exactly one field",
  },
  {
    code: "nitrogql2014",
    description: "Field of a @oneOf input object is null.",
    example: "Field 'id' of @oneOf input object must not be null",
  },
  {
    code: "nitrogql2015",
    description:
      "Variable used for a field of a @oneOf input object is nullable.",
    example:
      "Variable '$id' used for a field of @oneOf input object must be of non-null type",
  },
  {
    code: "nitrogql2016",
    description: "Output type is used where an input type is expected.",
    example: "Output type 'User' is not allowed here",
  },
  {
    code: "nitrogql2017",
    description: "Input type is used where an output type is expected.",
    example: "Input type 'UserInput' is not allowed here",
  },
];

const schemaErrors: readonly ErrorCodeEntry[] = [
  {
    code: "nitrogql3001",
    description: "Name that starts with __ is reserved.",
    example: "Name that starts with '__' is reserved",
  },
  {
    code: "nitrogql3002",
    description: "Name is duplicated.",
    example: "Name 'id' is duplicated",
  },
  {
    code: "nitrogql3003",
    description: "Type, directive or schema is declared more than once.",
    example: "Duplicated declaration of type 'User'",
  },
  {
    code: "nitrogql3004",
    description: "Extension has no original declaration.",
    example: "type is extended, but there is no original declaration of type",
  },
  {
    code: "nitrogql3005",
    description: "Directive references itself.",
    example: "Directive 'loop' is recursing",
  },
  {
    code: "nitrogql3006",
    description: "Implemented type is not an interface.",
    example: "'User' is not an interface",
  },
  {
    code: "nitrogql3007",
    description:
      "Type does not implement an interface that its interfaces implement.",
    example: "This type must implement interface 'Node'",
  },
  {
    code: "nitrogql3008",
    description: "Interface implements itself.",
    example: "Interface must not implement itself",
  },
  {
    code: "nitrogql3009",
    description: "Type does not have a field of its interface.",
    example: "This type must have a field 'id' from interface 'Node'",
  },
  {
    code: "nitrogql3010",
    description: "Type of a field does not match the field of the interface.",
    example:
      "Type of this field does not match the same field from interface 'Node'",
  },
  {
    code: "nitrogql3011",
    description:
      "Field does not have an argument of the same field of the interface.",
    example: "This field must have an argument 'id' from interface 'Node'",
  },
  {
    code: "nitrogql3012",
    description:
      "Type of an argument does not match the argument of the interface.",
    example:
      "Type of this argument does not match the same argument from interface 'Node'",
  },
  {
    code: "nitrogql3013",
    description:
      "Argument that the field of the interface does not have is of non-null type.",
    example:
      "Type of this argument must be nullable because it is not in the same field from interface 'Node'",
  },
  {
    code: "nitrogql3014",
    description: "Member of a union is not an object type.",
    example: "'Node' is not an object type",
  },
  {
    code: "nitrogql3015",
    description: "Field of a @oneOf input object is of non-null type.",
    example: "Field of @oneOf input object must not be of non-null type",
  },
  {
    code: "nitrogql3016",
    description: "Field of a @oneOf input object has a default value.",
    example: "Field of @oneOf input object must not have a default value",
  },
];

const operationErrors: readonly ErrorCodeEntry[] = [
  {
    code: "nitrogql4001",
    description: "Unnamed operation is not the only operation in the document.",
    example: "Unnamed operation must be the only operation in this document",
  },
  {
    code: "nitrogql4002",
    description: "Operation name is duplicated.",
    example: "Duplicate query name",
  },
  {
    code: "nitrogql4003",
    description: "Fragment name is duplicated.",
    example: "Duplicate fragment name",
  },
  {
    code: "nitrogql4004",
    description: "Root type for the operation is not defined.",
    example: "Root type for mutation operation is not defined",
  },
  {
    code: "nitrogql4005",
    description: "Fields are selected on a type that does not have fields.",
    example: "Cannot select fields of scalar 'Int'",
  },
  {
    code: "nitrogql4006",
    description: "Selection set is missing for a field of an object type.",
    example: "Must specify selection set for field 'me'",
  },
  {
    code: "nitrogql4007",
    description: "Field is not found on the type.",
    example: "Field 'nam' is not found on type 'User'",
  },
  {
    code: "nitrogql4008",
    description: "Variable name is duplicated.",
    example: "Duplicated variable name 'id'",
  },
  {
    code: "nitrogql4009",
    description: "Type is not a valid fragment target.",
    example: "Type 'Int' is not a valid fragment target",
  },
  {
    code: "nitrogql4010",
    description: "Fragment is not defined.",
    example: "Fragment 'Missing' is not defined",
  },
  {
    code: "nitrogql4011",
    description: "Fragment condition never matches its scope type.",
    example: "Fragment condition 'Query' never matches scope type 'User'",
  },
  {
    code: "nitrogql4012",
    description: "Fragment spreads itself.",
    example: "Recursing application of fragment 'UserParts'",
  },
  {
    code: "nitrogql4013",
    description: "Subscription operation does not have exactly one root field.",
    example: "Subscription operation must have exactly one root field",
  },
];

const importErrors: readonly ErrorCodeEntry[] = [
  {
    code: "nitrogql5001",
    description: "Wildcard import is specified more than once.",
    example: "Wildcard import should be specified only once",
  },
  {
    code: "nitrogql5002",
    description: "Wildcard import is combined with specific imports.",
    example: "Wildcard import cannot be combined with specific import",
  },
  {
    code: "nitrogql5003",
    description: "Imported file is not found.",
    example: "File './fragments.graphql' not found.",
  },
  {
    code: "nitrogql5004",
    description: "Imported fragment is not found in the imported file.",
    example:
      "'UserParts' is not found in the imported file './fragments.graphql'.",
  },
];

const pluginErrors: readonly ErrorCodeEntry[] = [
  {
    code: "nitrogql8001",
    description:
      "Error reported by a plugin. The message is provided by the plugin.",
    example: "",
  },
];

const otherErrors: readonly ErrorCodeEntry[] = [
  {
    code: "nitrogql9001",
    description: "Invalid option of the TypeScript content mapper.",
    example: "Unknown option 'configFiles'.",
  },
  {
    code: "nitrogql9002",
    description:
      "Problem outside GraphQL documents, such as in the config file. The content mapper also uses this code for errors in other files that prevent loading the project.",
    example: "Failed to load nitrogql project: Type 'Usr' is not defined",
  },
  {
    code: "nitrogql9999",
    description:
      "Internal error of nitrogql. If you see this error, please report it as a bug.",
    example: "Cannot merge selections of field 'a' that have different shapes",
  },
];

const ErrorCodeTable: React.FC<{ entries: readonly ErrorCodeEntry[] }> = ({
  entries,
}) => (
  <table>
    <thead>
      <tr>
        <th>Code</th>
        <th>Description</th>
        <th>Example message</th>
      </tr>
    </thead>
    <tbody>
      {entries.map(({ code, description, example }) => (
        <tr key={code} id={code}>
          <td>
            <code>{code}</code>
          </td>
          <td>{description}</td>
          <td>{example}</td>
        </tr>
      ))}
    </tbody>
  </table>
);

export default function ErrorCodes() {
  return (
    <Toc>
      <main>
        <Breadcrumb
          parents={[{ label: "References", href: "/references" }]}
          current="Error codes"
        />
        <h2>Error codes</h2>
        <p>
          Each error found in GraphQL documents has a code such as{" "}
          <code>nitrogql2001</code>. Codes are shown in the output of the{" "}
          <Link href="/cli">CLI</Link> and in diagnostics of the{" "}
          <Link href="/guides/typescript-content-mapper">
            TypeScript content mapper
          </Link>
          . A code never changes its meaning once published, while messages may
          change between versions.
        </p>
        <p>
          Codes are grouped by area: 1xxx for syntax errors, 2xxx for errors
          found in both schemas and operations, 3xxx for schemas, 4xxx for
          operations, 5xxx for <code>#import</code>, 8xxx for plugins, and 9xxx
          for everything else.
        </p>

        <h3 id="syntax-errors">Syntax errors</h3>
        <ErrorCodeTable entries={syntaxErrors} />

        <h3 id="common-errors">Errors in schemas and operations</h3>
        <ErrorCodeTable entries={commonErrors} />

        <h3 id="schema-errors">Errors in schemas</h3>
        <ErrorCodeTable entries={schemaErrors} />

        <h3 id="operation-errors">Errors in operations</h3>
        <ErrorCodeTable entries={operationErrors} />

        <h3 id="import-errors">
          Errors in <code>#import</code>
        </h3>
        <p>
          See <Link href="/references/syntax-import">The #import syntax</Link>{" "}
          for details.
        </p>
        <ErrorCodeTable entries={importErrors} />

        <h3 id="plugin-errors">Errors from plugins</h3>
        <ErrorCodeTable entries={pluginErrors} />

        <h3 id="other-errors">Other errors</h3>
        <ErrorCodeTable entries={otherErrors} />
      </main>
    </Toc>
  );
}
