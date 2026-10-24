#!/usr/bin/env node

// Content mapper for TypeScript 7.1+.
// TypeScript runs this script and communicates with it over stdio.

import { env, cwd, platform, stdin, stdout, exit } from "node:process";
import { parse, resolve } from "node:path";
import { runWasm } from "../lib/run-wasm.mjs";

const CWD = cwd();
// Project files can be anywhere, so the whole file system is exposed by default.
const NITROGQL_FS_SCOPE = env.NITROGQL_FS_SCOPE
  ? resolve(CWD, env.NITROGQL_FS_SCOPE)
  : parse(CWD).root;

const preopens = { [NITROGQL_FS_SCOPE]: NITROGQL_FS_SCOPE };
if (platform === "win32") {
  // Inside the wasm, a preopen only matches paths that start with its name
  // followed by `/`. TypeScript sends paths like `C:/foo/bar.graphql`, so
  // `C:\` must be exposed as `C:/`. Drive letters may come in either case
  // (VS Code uses lowercase).
  const name = NITROGQL_FS_SCOPE.replaceAll("\\", "/");
  for (const variant of [
    name,
    name.charAt(0).toUpperCase() + name.slice(1),
    name.charAt(0).toLowerCase() + name.slice(1),
  ]) {
    preopens[variant] = NITROGQL_FS_SCOPE;
  }
}

await runWasm({
  args: ["nitrogql", "content-mapper"],
  env: {
    ...env,
    CLICOLOR_FORCE: "0",
    NO_COLOR: "1",
    CWD,
  },
  preopens,
  contentMapper: {
    input: stdin,
    output: stdout,
    onClose: () => exit(0),
  },
});
