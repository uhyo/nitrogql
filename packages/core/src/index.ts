import { config } from "./config.js";
import { NitrogqlConfig } from "./configFormat.js";
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
};

export type { NitrogqlConfig };
