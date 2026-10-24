// Starts the nitrogql CLI wasm module. Shared by the entry points in `bin`.

import { readFile } from "node:fs/promises";
import * as core from "@nitrogql/core";
import { initWASI } from "@nitrogql/wasi-preview1";

/**
 * Instantiates the CLI wasm module and runs it.
 *
 * @param {object} options
 * @param {readonly string[]} options.args Command-line arguments, including the program name.
 * @param {Record<string, string>} options.env
 * @param {Record<string, string>} options.preopens
 * @param {Parameters<typeof core.initContentMapperNamespace>[0]} [options.contentMapper]
 *   Stdio connection to TypeScript. Only given in content mapper mode.
 */
export async function runWasm({ args, env, preopens, contentMapper }) {
  const wasi = initWASI({ args, env, preopens });
  const configHelper = core.initConfigNamespace();
  const contentMapperHelper =
    contentMapper && core.initContentMapperNamespace(contentMapper);

  const importObject = {
    wasi_snapshot_preview1: wasi,
    "nitrogql_helper/config": configHelper.namespace,
    // The same wasm module is used in every mode, so this namespace must
    // always be provided.
    "nitrogql_helper/content_mapper": contentMapperHelper?.namespace ?? {
      read_message() {
        throw new Error("Content mapper is not available in this mode");
      },
      write_message() {
        throw new Error("Content mapper is not available in this mode");
      },
    },
  };

  const wasm = await WebAssembly.compile(
    await readFile(new URL("../wasm/nitrogql-cli.wasm", import.meta.url)),
  );
  const instance = await WebAssembly.instantiate(wasm, importObject);
  wasi.setMemory(instance.exports.memory);
  core.setMemory(instance.exports.memory);
  configHelper.setWasmModule(instance.exports);
  contentMapperHelper?.setWasmModule(instance.exports);

  instance.exports._start();
}
