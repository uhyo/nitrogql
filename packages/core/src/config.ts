/**
 * @file `nitrogql_helper/config` namespace.
 */

import { execFileSync } from "node:child_process";
import { getMemory, readString } from "./memory.js";

/**
 * Executes given config file.
 * Note: requires `esbuild-register` and `esbuild` to be installed.
 * @returns result in JSON string
 */
export function executeConfigFileSync(configFilePath: string): string {
  return execFileSync(
    "node",
    [
      "--no-warnings",
      "--loader=esbuild-register/loader",
      "--require=esbuild-register",
      "--input-type=module",
    ],
    {
      encoding: "utf-8",
      input: `
import config from ${JSON.stringify(configFilePath)};
import { stdout } from "process";
stdout.write(JSON.stringify(config.default ?? config));
`,
    }
  );
}

export type NitrogqlConfigNamespace = {
  /**
   * Executes given config file.
   * Returns the handle to the result.
   * 0 if there was an error.
   */
  execute_config_file(
    config_file_path: number,
    config_file_path_len: number
  ): number;
  /**
   * Returns the length of the result.
   */
  result_len(handle: number): number;
  /**
   * Writes result of executing config file to given buffer.
   * Returns the number of bytes written.
   */
  write_result_to_buffer(
    handle: number,
    buffer: number,
    buffer_len: number
  ): number;
  /**
   * Frees the result.
   */
  free_result(handle: number): void;
};

/**
 * `nitrogql_helper/config` namespace
 * depended on by nitrogql's wasm modules.
 */
export const config: NitrogqlConfigNamespace = {
  execute_config_file,
  result_len,
  write_result_to_buffer,
  free_result,
};

const handleMap = new Map<number, string>();
let handleCounter = 0;

function execute_config_file(
  config_file_path: number,
  config_file_path_len: number
): number {
  const configFilePath = readString(config_file_path, config_file_path_len);
  try {
    const result = executeConfigFileSync(configFilePath);
    const handle = ++handleCounter;
    handleMap.set(handle, result);
    return handle;
  } catch (err) {
    console.error(err);
    return 0;
  }
}

function result_len(handle: number): number {
  const result = handleMap.get(handle);
  if (result === undefined) {
    return 0;
  }
  return result.length;
}

function write_result_to_buffer(
  handle: number,
  buffer: number,
  buffer_len: number
): number {
  const result = handleMap.get(handle);
  if (result === undefined) {
    return 0;
  }
  const bytes = new TextEncoder().encode(result);
  const memory = getMemory();
  const bytesToWrite = Math.min(bytes.length, buffer_len);
  const bufferView = new Uint8Array(memory.buffer, buffer, bytesToWrite);
  bufferView.set(bytes.slice(0, bytesToWrite));
  return bytesToWrite;
}

function free_result(handle: number): void {
  handleMap.delete(handle);
}
