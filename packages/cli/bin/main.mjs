#!/usr/bin/env node

import { argv, env, cwd, stdout } from "node:process";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import * as core from "@nitrogql/core";
import { initWASI } from "@nitrogql/wasi-preview1";

const CWD = cwd();
const NITROGQL_FS_SCOPE = env.NITROGQL_FS_SCOPE
  ? resolve(CWD, env.NITROGQL_FS_SCOPE)
  : CWD;
const isTTY = stdout.isTTY;

const wasi = initWASI({
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

const configHelper = core.initConfigNamespace();

const importObject = {
  wasi_snapshot_preview1: wasi,
  "nitrogql_helper/config": configHelper.namespace,
};

const wasm = await WebAssembly.compile(
  await readFile(new URL("../wasm/nitrogql-cli.wasm", import.meta.url))
);
const instance = await WebAssembly.instantiate(wasm, importObject);
wasi.setMemory(instance.exports.memory);
core.setMemory(instance.exports.memory);
configHelper.setWasmModule(instance.exports);

instance.exports._start();
