#! /usr/bin/env node --experimental-wasi-unstable-preview1

import { WASI } from 'node:wasi';
import { argv, env, cwd } from 'node:process';
import { readFile } from 'node:fs/promises';
import { StringAllocator } from './alloc.mjs';

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

const importObject = { wasi_snapshot_preview1: wasi.wasiImport };

const wasm = await WebAssembly.compile(
  await readFile(new URL('../wasm/graphql-loader.wasm', import.meta.url)),
);
const instance = await WebAssembly.instantiate(wasm, importObject);

wasi.initialize(instance);

const alloc = new StringAllocator(instance);

instance.exports.init();

instance.exports.load_config(0, 0);

const inputString = alloc.allocString(`query { a b c }`);

const convertResult = instance.exports.convert_to_js(inputString.ptr, inputString.size);

inputString.free();

if (convertResult) {
  const ptr = instance.exports.get_result_ptr();
  const size = instance.exports.get_result_size();
  const result = alloc.readString(ptr, size);
  console.log({ result })
}