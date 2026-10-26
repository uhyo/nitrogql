// Measures the cost of fully resolving result types of a case, for expanded and referred modules.
// Usage: node measure.cjs [case]
const path = require("node:path");
const ts = require(process.env.TYPESCRIPT_API ?? "typescript6");

const dir = path.join(__dirname, "generated", process.argv[2] ?? "stress");
// Instantiations of an empty program, which are subtracted from the counts.
const baseline = (() => {
  const program = ts.createProgram(
    [path.join(__dirname, "generated", "typed-document-node.ts")],
    { strict: true, noEmit: true, types: [] },
  );
  ts.getPreEmitDiagnostics(program);
  return program.getTypeChecker().getInstantiationCount();
})();
for (const file of ["expanded.ts", "main.ts"]) {
  const start = performance.now();
  const program = ts.createProgram([path.join(dir, file)], {
    strict: true,
    noEmit: true,
    types: [],
  });
  const checker = program.getTypeChecker();
  const source = program.getSourceFile(path.join(dir, file));
  let length = 0;
  for (const statement of source.statements) {
    if (
      ts.isTypeAliasDeclaration(statement) &&
      statement.name.text.endsWith("Result")
    ) {
      const type = checker.getTypeAtLocation(statement.name);
      length += checker.typeToString(
        type,
        undefined,
        ts.TypeFormatFlags.NoTruncation,
      ).length;
    }
  }
  const diagnostics = ts.getPreEmitDiagnostics(program).length;
  console.log(
    `${file.padEnd(12)} time=${(performance.now() - start).toFixed(0)}ms types=${checker.getTypeCount()} ` +
      `instantiations=${checker.getInstantiationCount() - baseline} printed=${length} diagnostics=${diagnostics}`,
  );
}
