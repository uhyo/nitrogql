// @ts-check

import path from "node:path";
import { readFile } from "node:fs/promises";
import { createFilter } from "@rollup/pluginutils";
import { StringAllocator } from "./alloc.mjs";

const wasm = await WebAssembly.compile(
  await readFile(new URL("../wasm/graphql-loader.wasm", import.meta.url))
);
const instance = await WebAssembly.instantiate(wasm);

const alloc = new StringAllocator(instance);

instance.exports.init();

let lastLoadedConfigPath = undefined;

/**
 * @type {import('rollup').PluginImpl<{
 *   configFile?: string;
 *   include?: string | readonly string[];
 *   exclude?: string | readonly string[];
 * }>}
 */
export default function NitrogqlRollupPlugin(options) {
  const { configFile, include, exclude } = options ?? {};
  const filter = createFilter(include, exclude);
  let root = process.cwd();

  return {
    name: "@nitrogql/rollup-plugin",
    // Vite-specific hook to resolve the root directory of the project.
    // @ts-expect-error
    configResolved(config) {
      root = config.root;
    },
    async transform(source, id) {
      if (!filter(id)) {
        return;
      }

      const configFilePath = configFile && path.resolve(root, configFile);
      if (configFilePath) {
        this.addWatchFile(configFilePath);
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
    },
  };
}

function configFileIsJS(configFile) {
  return /\.[cm]?js$/.test(configFile);
}
