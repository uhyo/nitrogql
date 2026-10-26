// Prints expanded and referred types of each case as TypeScript displays them.
// Usage: node dump-types.cjs [case...]
const path = require("node:path");
const fs = require("node:fs");
const ts = require(process.env.TYPESCRIPT_API ?? "typescript");

const generated = path.join(__dirname, "generated");
const cases =
  process.argv.length > 2
    ? process.argv.slice(2)
    : fs
        .readdirSync(generated)
        .filter((name) =>
          fs.existsSync(path.join(generated, name, "check.ts")),
        );
const files = cases.flatMap((name) =>
  ["expanded.ts", "main.ts"].map((file) => path.join(generated, name, file)),
);
const program = ts.createProgram(files, {
  strict: true,
  noEmit: true,
  types: [],
});
const checker = program.getTypeChecker();
const flags =
  ts.TypeFormatFlags.NoTruncation |
  ts.TypeFormatFlags.InTypeAlias |
  ts.TypeFormatFlags.MultilineObjectLiterals;

for (const name of cases) {
  console.log(`=== ${name}`);
  for (const file of ["expanded.ts", "main.ts"]) {
    const source = program.getSourceFile(path.join(generated, name, file));
    for (const statement of source.statements) {
      if (
        !ts.isTypeAliasDeclaration(statement) ||
        statement.name.text.startsWith("__")
      )
        continue;
      if (
        !statement.modifiers?.some(
          (m) => m.kind === ts.SyntaxKind.ExportKeyword,
        )
      )
        continue;
      if (statement.name.text.endsWith("Variables")) continue;
      const type = checker.getTypeAtLocation(statement.name);
      console.log(
        `--- ${file} ${statement.name.text}\n${checker.typeToString(type, undefined, flags)}`,
      );
    }
  }
}
