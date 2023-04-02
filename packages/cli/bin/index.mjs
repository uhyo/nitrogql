#! /usr/bin/env node --no-warnings --experimental-wasi-unstable-preview1

import { WASI } from "node:wasi";
import { argv, env, cwd, stdout } from "node:process";
import { readFile } from "node:fs/promises";
import { shim } from "./shim.mjs";

const CWD = cwd();
const isTTY = stdout.isTTY;

const wasi = new WASI({
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
    [CWD]: CWD,
  },
});

let memoryRef = { memory: null };
const importObject = {
  wasi_snapshot_preview1: {
    ...wasi.wasiImport,
    ...shim(wasi.wasiImport, memoryRef, CWD),
  },
};

const wasm = await WebAssembly.compile(
  await readFile(new URL("../wasm/nitrogql-cli.wasm", import.meta.url))
);
const instance = await WebAssembly.instantiate(wasm, importObject);
memoryRef.memory = instance.exports.memory;

wasi.start(instance);
