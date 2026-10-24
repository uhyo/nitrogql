#!/usr/bin/env node

// Content mapper for TypeScript 7.1+.
// TypeScript runs this script and communicates with it over stdio.

import { env, cwd, stdin, stdout, exit } from "node:process";
import { parse, resolve } from "node:path";
import { runWasm } from "../lib/run-wasm.mjs";

const CWD = cwd();
// Project files can be anywhere, so the whole file system is exposed by default.
const NITROGQL_FS_SCOPE = env.NITROGQL_FS_SCOPE
  ? resolve(CWD, env.NITROGQL_FS_SCOPE)
  : parse(CWD).root;

await runWasm({
  args: ["nitrogql", "content-mapper"],
  env: {
    ...env,
    CLICOLOR_FORCE: "0",
    NO_COLOR: "1",
    CWD,
  },
  preopens: {
    [NITROGQL_FS_SCOPE]: NITROGQL_FS_SCOPE,
  },
  contentMapper: {
    input: stdin,
    output: stdout,
    onClose: () => exit(0),
  },
});
