#!/usr/bin/env node
// Compares the type checking cost of imported fragments with that of inlined fragments.
//
// The content mapper does not expand `#import`ed fragments; their types are merged by
// TypeScript instead. To measure the cost of this, the project is copied twice:
//
// - imported: as is.
// - inlined: `#import` statements are removed and definitions of the imported fragments
//   (and fragments they depend on, transitively) are appended to each file that needs them.
//
// Then both copies are type-checked with `tsc --runExternalCode --extendedDiagnostics`.
//
// Usage: node scripts/bench-fragments.mjs [--runs N] [--out DIR]

import { spawnSync } from "node:child_process";
import {
  cpSync,
  existsSync,
  mkdirSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { dirname, join, relative, resolve } from "node:path";
import { argv, execPath, exit } from "node:process";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";
import { Kind, parse, visit } from "graphql";

const projectDir = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const require = createRequire(import.meta.url);

const { values: options } = parseArgs({
  args: argv.slice(2),
  options: {
    runs: { type: "string", default: "3" },
    out: {
      type: "string",
      default: join(tmpdir(), "nitrogql-bench-fragments"),
    },
  },
});
const runs = Number(options.runs);
const outDir = resolve(options.out);

const variants = {
  imported: join(outDir, "imported"),
  inlined: join(outDir, "inlined"),
};
rmSync(outDir, { recursive: true, force: true });
for (const dir of Object.values(variants)) {
  copyProject(dir);
}
const stats = inlineFragments(variants.inlined);
console.log(
  `Inlined ${stats.definitions} fragment definitions into ${stats.files} files (${stats.imports} #import statements removed).`,
);
console.log(`Projects are in ${outDir}\n`);

const results = {};
for (const [name, dir] of Object.entries(variants)) {
  results[name] = [];
  // The first run in a new directory can be slow to start the content mapper, so it is
  // not measured. It may even fail when TypeScript gives up waiting for the content mapper.
  typeCheck(dir, { warmUp: true });
  for (let i = 0; i < runs; i++) {
    results[name].push(typeCheck(dir));
  }
}
printTable(results);

/** Copies files needed for type checking. Dependencies are resolved from the repository. */
function copyProject(dir) {
  mkdirSync(dir, { recursive: true });
  for (const file of [
    "src",
    "tsconfig.json",
    "graphql.config.yaml",
    "next-env.d.ts",
  ]) {
    const source = join(projectDir, file);
    if (existsSync(source)) {
      cpSync(source, join(dir, file), { recursive: true });
    }
  }
  if (!existsSync(join(dir, "src/generated"))) {
    throw new Error("Run `npm run generate` first.");
  }
  // Packages are hoisted to the repository root.
  symlinkSync(
    resolve(projectDir, "../../node_modules"),
    join(dir, "node_modules"),
    "dir",
  );
}

/**
 * Replaces `#import` statements with definitions of the fragments they provide.
 * Fragment names must be unique in the project, as `nitrogql check` requires.
 */
function inlineFragments(dir) {
  const files = new Map(
    globDocuments(dir).map((path) => [path, parseDocument(path)]),
  );
  const stats = { files: 0, definitions: 0, imports: 0 };

  /** Finds the definition of a fragment as seen from a file. */
  const resolveFragment = (file, name) => {
    const local = file.fragments.get(name);
    if (local) {
      return { file, definition: local };
    }
    for (const { targets, path } of file.imports) {
      if (targets === "*" || targets.includes(name)) {
        const imported = files.get(path);
        const definition = imported?.fragments.get(name);
        if (definition) {
          return { file: imported, definition };
        }
      }
    }
    throw new Error(`Cannot resolve fragment ${name} in ${file.path}`);
  };

  for (const file of files.values()) {
    if (file.imports.length === 0) {
      continue;
    }
    // Definitions from other files, in order of discovery.
    const inlined = new Map();
    const visitSpreads = (from, definition) => {
      for (const name of spreadsOf(definition)) {
        const found = resolveFragment(from, name);
        if (found.file === file || inlined.has(name)) {
          continue;
        }
        inlined.set(name, found);
        visitSpreads(found.file, found.definition);
      }
    };
    for (const definition of file.document.definitions) {
      visitSpreads(file, definition);
    }
    const source = file.source.replace(/^#import .*\n/gm, "").trimEnd();
    const appended = [...inlined.values()].map(
      ({ file, definition }) =>
        `# Inlined from ${relative(dir, file.path)}\n` +
        file.source.slice(definition.loc.start, definition.loc.end),
    );
    writeFileSync(file.path, [source, ...appended].join("\n\n") + "\n");
    stats.files++;
    stats.definitions += inlined.size;
    stats.imports += file.imports.length;
  }
  return stats;
}

function parseDocument(path) {
  const source = readFileSync(path, "utf8");
  const document = parse(source);
  const imports = [...source.matchAll(/^#import (.+?) from "(.+)"$/gm)].map(
    ([, targets, specifier]) => ({
      targets:
        targets.trim() === "*"
          ? "*"
          : targets.split(",").map((target) => target.trim()),
      path: resolve(dirname(path), specifier),
    }),
  );
  const fragments = new Map(
    document.definitions
      .filter((definition) => definition.kind === Kind.FRAGMENT_DEFINITION)
      .map((definition) => [definition.name.value, definition]),
  );
  return { path, source, document, imports, fragments };
}

function spreadsOf(definition) {
  const names = [];
  visit(definition, {
    FragmentSpread(node) {
      names.push(node.name.value);
    },
  });
  return names;
}

function globDocuments(dir) {
  const result = spawnSync(
    "git",
    ["ls-files", "--others", "--cached", "--", "*.graphql"],
    { cwd: projectDir, encoding: "utf8" },
  );
  return result.stdout
    .split("\n")
    .filter((path) => /^src\/(app|components)\//.test(path))
    .map((path) => join(dir, path));
}

function typeCheck(dir, { warmUp = false } = {}) {
  const tsc = join(
    dirname(require.resolve("typescript7/package.json")),
    "bin/tsc",
  );
  const { stdout, status } = spawnSync(
    execPath,
    [
      tsc,
      "-p",
      dir,
      "--runExternalCode",
      "--extendedDiagnostics",
      "--incremental",
      "false",
    ],
    { cwd: dir, encoding: "utf8" },
  );
  if (warmUp) {
    return;
  }
  if (status !== 0) {
    console.error(stdout);
    console.error(`Type checking failed in ${dir}`);
    exit(1);
  }
  const metric = (name) =>
    stdout.match(new RegExp(`^${name}:\\s+([\\d.]+)`, "m"))?.[1];
  return {
    files: Number(metric("Files")),
    types: Number(metric("Types")),
    instantiations: Number(metric("Instantiations")),
    checkTime: Number(metric("Check time")),
    mapperTime: Number(metric("Content mapper request wait time")),
    totalTime: Number(metric("Total time")),
  };
}

function printTable(results) {
  const median = (values) =>
    [...values].sort((a, b) => a - b)[Math.floor(values.length / 2)];
  const rows = Object.entries(results).map(([name, runs]) => [
    name,
    String(runs[0].files),
    runs[0].types.toLocaleString("en"),
    runs[0].instantiations.toLocaleString("en"),
    `${median(runs.map((run) => run.checkTime)).toFixed(2)}s`,
    `${median(runs.map((run) => run.mapperTime)).toFixed(2)}s`,
    `${median(runs.map((run) => run.totalTime)).toFixed(2)}s`,
  ]);
  const header = [
    "Fragments",
    "Files",
    "Types",
    "Instantiations",
    "Check time",
    "Content mapper wait",
    "Total time",
  ];
  const widths = header.map((title, i) =>
    Math.max(title.length, ...rows.map((row) => row[i].length)),
  );
  const line = (cells) =>
    `| ${cells.map((cell, i) => cell.padEnd(widths[i])).join(" | ")} |`;
  console.log(line(header));
  console.log(line(widths.map((width) => "-".repeat(width))));
  for (const row of rows) {
    console.log(line(row));
  }
  console.log(`\nTimes are medians of ${runs} runs.`);
}
