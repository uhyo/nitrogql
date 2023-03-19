#! /usr/bin/env node --experimental-wasi-unstable-preview1

import path from "node:path";
import { WASI } from "node:wasi";
import { argv, env, cwd } from "node:process";
import { readFile } from "node:fs/promises";
import { StringAllocator } from "./alloc.mjs";

const CWD = cwd();

const wasi = new WASI({
  args: argv,
  env: {
    ...env,
    CWD,
  },
  preopens: {
    [CWD]: CWD,
  },
});

const importObject = { wasi_snapshot_preview1: wasi.wasiImport };

const wasm = await WebAssembly.compile(
  await readFile(new URL("../wasm/graphql-loader.wasm", import.meta.url))
);
const instance = await WebAssembly.instantiate(wasm, importObject);

wasi.initialize(instance);

const alloc = new StringAllocator(instance);

instance.exports.init();

let lastLoadedConfigPath = undefined;

export default function graphQLLoader(source) {
  const options = this.getOptions();
  const configFile = options?.configFile;
  if (lastLoadedConfigPath !== configFile && typeof configFile === "string") {
    const configFilePath = path.resolve(this.rootContext, configFile);
    const configFilePathString = alloc.allocString(configFilePath);
    instance.exports.load_config(
      configFilePathString.ptr,
      configFilePathString.size
    );
    configFilePathString.free();
  }

  const inputString = alloc.allocString(source);

  const convertResult = instance.exports.convert_to_js(
    inputString.ptr,
    inputString.size
  );
  inputString.free();
  if (convertResult) {
    const ptr = instance.exports.get_result_ptr();
    const size = instance.exports.get_result_size();
    const result = alloc.readString(ptr, size);
    return result;
  } else {
    throw new Error("graphql-loader failed to convert");
  }
}
