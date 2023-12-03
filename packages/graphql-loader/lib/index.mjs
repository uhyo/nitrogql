// @ts-check

import path from "node:path";
import { readFile } from "node:fs/promises";
import { executeConfigFileSync } from "@nitrogql/core";
import { StringAllocator } from "./alloc.mjs";

class WasmError extends Error {}

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
        ? executeConfigFileSync(configFilePath)
        : await readFile(configFilePath, "utf-8");
      const configFilePathString = alloc.allocString(configFileSource);
      instance.exports.load_config(
        configFilePathString.ptr,
        configFilePathString.size
      );
      configFilePathString.free();
    }
    lastLoadedConfigPath = configFilePath;

    const filenameString = alloc.allocString(this.resourcePath);
    const inputString = alloc.allocString(source);
    console.debug("initialize_task", this.resourcePath);
    const taskId = instance.exports.initiate_task(
      filenameString.ptr,
      filenameString.size,
      inputString.ptr,
      inputString.size
    );
    if (taskId === 0) {
      throw new WasmError("graphql-loader failed to initiate task");
    }

    // Load all required files
    while (true) {
      const getRequiredFilesResult =
        instance.exports.get_required_files(taskId);
      if (!getRequiredFilesResult) {
        throw new WasmError("graphql-loader failed to get required files");
      }
      const requiredFilesPtr = instance.exports.get_result_ptr();
      const requiredFilesSize = instance.exports.get_result_size();
      const requiredFiles = alloc
        .readString(requiredFilesPtr, requiredFilesSize)
        .split("\n")
        .filter(Boolean);
      if (requiredFiles.length === 0) {
        break;
      }
      await Promise.all(
        requiredFiles.map(async (requiredFile) => {
          this.addDependency(requiredFile);
          const requiredFileSource = await readFile(requiredFile, "utf-8");
          const requiredFileString = alloc.allocString(requiredFile);
          const requiredFileSourceString =
            alloc.allocString(requiredFileSource);
          console.debug("load_file", requiredFile, requiredFileSource);
          const loadFileResult = instance.exports.load_file(
            taskId,
            requiredFileString.ptr,
            requiredFileString.size,
            requiredFileSourceString.ptr,
            requiredFileSourceString.size
          );
          requiredFileString.free();
          requiredFileSourceString.free();
          if (!loadFileResult) {
            throw new WasmError(
              `graphql-loader failed to load file: ${requiredFile}`
            );
          }
        })
      );
    }

    const emitResult = instance.exports.emit_js(taskId);
    filenameString.free();
    inputString.free();
    instance.exports.free_task(taskId);
    if (emitResult) {
      const ptr = instance.exports.get_result_ptr();
      const size = instance.exports.get_result_size();
      const result = alloc.readString(ptr, size);
      return result;
    } else {
      throw new WasmError("graphql-loader failed to emit result");
    }
  })().then(
    (res) => callback(null, res),
    (err) => {
      if (err instanceof WasmError) {
        const ptr = instance.exports.get_result_ptr();
        const size = instance.exports.get_result_size();
        const errorMessage = alloc.readString(ptr, size);
        console.error(errorMessage);
      }
      callback(err);
    }
  );
}

function configFileIsJS(configFile) {
  return /\.[cm]?[jt]s$/.test(configFile);
}
