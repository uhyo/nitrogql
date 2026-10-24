#!/usr/bin/env node

// Content mapper for TypeScript 7.1+.
// TypeScript runs this script and communicates with it over stdio.

import { env, cwd, stdin, stdout, exit } from "node:process";
import { readFile } from "node:fs/promises";
import { parse, resolve } from "node:path";
import * as core from "@nitrogql/core";
import { initWASI } from "@nitrogql/wasi-preview1";

const CWD = cwd();
// Project files can be anywhere, so the whole file system is exposed by default.
const NITROGQL_FS_SCOPE = env.NITROGQL_FS_SCOPE
  ? resolve(CWD, env.NITROGQL_FS_SCOPE)
  : parse(CWD).root;

const wasi = initWASI({
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
});

const configHelper = core.initConfigNamespace();
const contentMapperHelper = core.initContentMapperNamespace({
  input: stdin,
  output: stdout,
  onClose: () => exit(0),
});

const importObject = {
  wasi_snapshot_preview1: wasi,
  "nitrogql_helper/config": configHelper.namespace,
  "nitrogql_helper/content_mapper": contentMapperHelper.namespace,
};

const wasm = await WebAssembly.compile(
  await readFile(new URL("../wasm/nitrogql-cli.wasm", import.meta.url)),
);
const instance = await WebAssembly.instantiate(wasm, importObject);
wasi.setMemory(instance.exports.memory);
core.setMemory(instance.exports.memory);
configHelper.setWasmModule(instance.exports);
contentMapperHelper.setWasmModule(instance.exports);

instance.exports._start();
