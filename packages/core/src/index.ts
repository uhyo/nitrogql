import { config, executeConfigFileSync, executeNodeSync } from "./config.js";
import { NitrogqlConfig } from "./configFormat.js";
import { loadSchemaJs } from "./loader.js";
import { setMemory } from "./memory.js";

export {
  /**
   * Sets the memory used by the wasm module.
   */
  setMemory,
  /**
   * `nitrogql_helper/config` namespace
   */
  config,
  executeNodeSync,
  executeConfigFileSync,
  loadSchemaJs,
};

export type { NitrogqlConfig };
