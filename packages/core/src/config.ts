/**
 * @file `nitrogql_helper/config` namespace.
 */

import { execFileSync } from "node:child_process";
import { getMemory, readString } from "./memory.js";

/**
 * Executes given code with Node.js.
 * Requires @nitrogql/esbuild-register to be installed.
 * @returns stdout
 */
export function executeNodeSync(code: string): string {
  const nodeVersion = process.versions.node;
  // @nitrogql/esbuild-register requires different usage
  // depending on whether Node.js >= 20.6.0 or not.
  const [major, minor] = nodeVersion.split(".").map((x) => Number(x)) as [
    number,
    number
  ];
  const isNode2060OrLater = major > 20 || (major === 20 && minor >= 6);
  if (isNode2060OrLater) {
    return execFileSync(
      process.execPath,
      [
        "--no-warnings",
        "--import=@nitrogql/esbuild-register",
        "--input-type=module",
      ],
      {
        encoding: "utf-8",
        input: code,
      }
    );
  }
  return execFileSync(
    process.execPath,
    [
      "--no-warnings",
      "--import=@nitrogql/esbuild-register",
      "--experimental-loader=@nitrogql/esbuild-register/hook",
      "--input-type=module",
    ],
    {
      encoding: "utf-8",
      input: code,
    }
  );
}

/**
 * Executes given config file.
 */
export function executeConfigFileSync(configFilePath: string): string {
  return executeNodeSync(`
import config from ${JSON.stringify(configFilePath)};
import { stdout } from "process";
stdout.write(JSON.stringify(config.default ?? config));
`);
}

export type NitrogqlConfigNamespace = {
  /**
   * Executes given JavaScript (or TypeScript) code.
   * Returns the handle to the result (string written to stdout).
   * 0 if there was an error.
   */
  execute_node(code_ptr: number, code_len: number): number;
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
  execute_node,
  result_len,
  write_result_to_buffer,
  free_result,
};

const handleMap = new Map<number, string>();
let handleCounter = 0;

function execute_node(code_ptr: number, code_len: number): number {
  const code = readString(code_ptr, code_len);
  try {
    const result = executeNodeSync(code);
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
