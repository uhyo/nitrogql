#! /usr/bin/env node --experimental-wasi-unstable-preview1

import { WASI } from 'node:wasi';
import { argv, env, cwd } from 'node:process';
import { readFile } from 'node:fs/promises';

const CWD = cwd();

const wasi = new WASI({
  args: argv,
  env: {
    ...env,
    CWD,
  },
  preopens: {
    [CWD]: CWD
  },
});

// Some WASI binaries require:
//   const importObject = { wasi_unstable: wasi.wasiImport };
const importObject = { wasi_snapshot_preview1: wasi.wasiImport };

const wasm = await WebAssembly.compile(
  await readFile(new URL('../wasm/nitrogql.wasi.wasm', import.meta.url)),
);
const instance = await WebAssembly.instantiate(wasm, importObject);

wasi.start(instance);