import { readFile } from "fs/promises";
import { WASMInstance } from "./wasm.js";
import { Task } from "./task.js";
import { WASMBin, WasmError } from "./bin.js";

/**
 * Initialize the loader binary.
 */
export async function init() {
  const wasm = await WebAssembly.compile(
    await readFile(new URL("../wasm/graphql-loader.wasm", import.meta.url))
  );
  const instance = (await WebAssembly.instantiate(wasm)) as WASMInstance;

  const bin = new WASMBin(instance);

  const debugFlag = !!process.env.NITROGQL_DEBUG;

  instance.exports.init(+debugFlag);

  return {
    /**
     * Initializes a task for converting one file.
     * @param fileName
     * @param source
     */
    initiateTask(fileName: string, source: string) {
      const filenameString = bin.alloc.allocString(fileName);
      const inputString = bin.alloc.allocString(source);
      try {
        const taskId = instance.exports.initiate_task(
          filenameString.ptr,
          filenameString.size,
          inputString.ptr,
          inputString.size
        );
        if (taskId === 0) {
          throw new WasmError("Failed to initiate task", bin);
        }
        return new Task(bin, taskId);
      } finally {
        filenameString.free();
        inputString.free();
      }
    },
    /**
     * Reads and consumes the log from the loader.
     */
    getLog(): string | undefined {
      if (!debugFlag) {
        return undefined;
      }
      instance.exports.get_log();
      const result = bin.readResult();
      return result;
    },
  };
}
