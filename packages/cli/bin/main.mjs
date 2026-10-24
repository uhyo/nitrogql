#!/usr/bin/env node

import { argv, env, cwd, stdout } from "node:process";
import { resolve } from "node:path";
import { runWasm } from "../lib/run-wasm.mjs";

const CWD = cwd();
const NITROGQL_FS_SCOPE = env.NITROGQL_FS_SCOPE
  ? resolve(CWD, env.NITROGQL_FS_SCOPE)
  : CWD;
const isTTY = stdout.isTTY;

await runWasm({
  args: argv.slice(1),
  env: {
    // env_logger
    RUST_LOG_STYLE: isTTY ? "always" : "auto",
    // colored
    CLICOLOR_FORCE: isTTY ? "1" : "0",
    ...env,
    CWD,
  },
  preopens: {
    [NITROGQL_FS_SCOPE]: NITROGQL_FS_SCOPE,
  },
});
