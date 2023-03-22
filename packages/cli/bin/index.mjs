#! /usr/bin/env node --experimental-wasi-unstable-preview1

import { WASI } from "node:wasi";
import { argv, env, cwd } from "node:process";
import { readFile } from "node:fs/promises";
import { shim } from "./shim.mjs";

const CWD = cwd();

const wasi = new WASI({
  args: argv.slice(1),
  env: {
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
