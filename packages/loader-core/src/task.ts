import { WasmError, type WASMBin } from "./bin.js";

export type TaskStatus =
  | {
      status: "fileRequired";
      /**
       * The file that is required.
       */
      files: readonly string[];
    }
  | {
      status: "ready";
    };

/**
 * Represents a task of converting one GraphQL file to JavaScript.
 */
export class Task {
  #bin: WASMBin;
  readonly taskId: number;
  constructor(bin: WASMBin, taskId: number) {
    this.#bin = bin;
    this.taskId = taskId;
  }

  /**
   * Load configuration from given JSON string.
   * TODO: in current implementation config is internally shared among all tasks.
   */
  loadConfig(configString: string) {
    const configFilePathString = this.#bin.alloc.allocString(configString);
    this.#bin.exports.load_config(
      configFilePathString.ptr,
      configFilePathString.size
    );
    configFilePathString.free();
  }

  /**
   * Returns the current status of the task.
   */
  status(): TaskStatus {
    const getRequiredFilesResult = this.#bin.exports.get_required_files(
      this.taskId
    );
    if (!getRequiredFilesResult) {
      throw new WasmError(
        "graphql-loader failed to get required files",
        this.#bin
      );
    }
    const requiredFiles = this.#bin.readResult().split("\n").filter(Boolean);
    if (requiredFiles.length === 0) {
      return { status: "ready" };
    }
    return { status: "fileRequired", files: requiredFiles };
  }

  /**
   * Supply one file to the task.
   */
  supplyFile(filePath: string, source: string) {
    const filePathString = this.#bin.alloc.allocString(filePath);
    const sourceString = this.#bin.alloc.allocString(source);
    try {
      const result = this.#bin.exports.load_file(
        this.taskId,
        filePathString.ptr,
        filePathString.size,
        sourceString.ptr,
        sourceString.size
      );
      if (!result) {
        throw new WasmError("graphql-loader failed to load file", this.#bin);
      }
    } finally {
      filePathString.free();
      sourceString.free();
    }
  }

  /**
   * Emit JavaScript.
   */
  emit() {
    const result = this.#bin.exports.emit_js(this.taskId);
    if (!result) {
      throw new WasmError(
        "graphql-loader failed to emit JavaScript",
        this.#bin
      );
    }
    const js = this.#bin.readResult();
    return js;
  }

  /**
   * Free the task.
   */
  free() {
    this.#bin.exports.free_task(this.taskId);
  }
}
