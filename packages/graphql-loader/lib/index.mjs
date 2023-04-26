// @ts-check

import path from "node:path";
import { readFile } from "node:fs/promises";
import { StringAllocator } from "./alloc.mjs";

const wasm = await WebAssembly.compile(
  await readFile(new URL("../wasm/graphql-loader.wasm", import.meta.url))
);
const instance = await WebAssembly.instantiate(wasm);

const alloc = new StringAllocator(instance);

instance.exports.init();

let lastLoadedConfigPath = undefined;

/**
 * @type {import('webpack').LoaderDefinitionFunction}
 */
export default function graphQLLoader(source) {
  const callback = this.async();
  (async () => {
    const options = this.getOptions();
    const configFile = options?.configFile;
    const configFilePath =
      configFile && path.resolve(this.rootContext, configFile);
    if (configFilePath) {
      this.addDependency(configFilePath);
    }

    if (lastLoadedConfigPath !== configFilePath && configFilePath) {
      const configFileSource = configFileIsJS(configFilePath)
        ? JSON.stringify(await import(configFilePath))
        : await readFile(configFilePath, "utf-8");
      const configFilePathString = alloc.allocString(configFileSource);
      instance.exports.load_config(
        configFilePathString.ptr,
        configFilePathString.size
      );
      configFilePathString.free();
    }
    lastLoadedConfigPath = configFilePath;

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
  })().then(
    (res) => callback(null, res),
    (err) => callback(err)
  );
}

function configFileIsJS(configFile) {
  return /\.[cm]?js$/.test(configFile);
}
