/**
 * @file `nitrogql_helper/config` namespace.
 */

import { execFileSync } from "node:child_process";
import { getMemory, readString, utf8Len, writeString } from "./memory.js";
import { getCommandClient } from "./command/commandClient.js";

/**
 * Executes given code with Node.js.
 * Requires @nitrogql/esbuild-register to be installed.
 * @returns stdout
 */
export function executeNodeSync(code: string): string {
  const nodeVersion = process.versions.node;
  // @nitrogql/esbuild-register requires different usage
  // depending on whether Node.js supports the `register` API from `node:module`.
  const [major, minor] = nodeVersion.split(".").map((x) => Number(x)) as [
    number,
    number
  ];
  const nodeHasModuleRegisterAPI =
    major > 20 || (major === 20 && minor >= 6) || (major === 18 && minor >= 19);
  if (nodeHasModuleRegisterAPI) {
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
      "--require=@nitrogql/esbuild-register",
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
   * Result is returned asynchronously via `execute_node_ret` function.
   */
  execute_node(
    code_ptr: number,
    code_len: number,
    ticket_handle: number
  ): number;
};

export type InitNitrogqlConfigResult = {
  namespace: NitrogqlConfigNamespace;
  setWasmModule: (module: WebAssembly.Exports) => void;
};

let handleCounter = 0;

/**
 * Initialize the `nitrogql_helper/config` namespace.
 * This namespace is depended by nitrogql's wasm modules.
 */
export function initConfigNamespace(): InitNitrogqlConfigResult {
  let module: WebAssembly.Exports | undefined = undefined;
  const w = getCommandClient();
  const namespace: NitrogqlConfigNamespace = {
    execute_node,
  };
  return {
    namespace,
    setWasmModule: (m) => {
      module = m;
    },
  };

  function execute_node(
    code_ptr: number,
    code_len: number,
    ticket_handle: number
  ): number {
    debugger;
    if (module === undefined) {
      throw new Error("wasm module is not set");
    }
    const m = module;
    const alloc_string = m.alloc_string as (len: number) => number;
    const free_string = m.free_string as (ptr: number, len: number) => void;
    const execute_node_ret = m.execute_node_ret as (
      ticket_handle: number,
      is_ok: number,
      result_ptr: number,
      result_len: number
    ) => void;
    const code = readString(code_ptr, code_len);
    const handle = ++handleCounter;
    w.run(code)
      .then((data) => {
        const result = JSON.stringify(data);
        // Write the result to the buffer.
        const len = utf8Len(result);
        const result_ptr = alloc_string(len);
        writeString(result, result_ptr, len);
        execute_node_ret(ticket_handle, 1, result_ptr, len);
        free_string(result_ptr, len);
      })
      .catch((error) => {
        console.error(error);
        execute_node_ret(ticket_handle, 0, 0, 0);
      });
    return handle;
  }
}
