import {
  initConfigNamespace,
  executeConfigFileSync,
  executeNodeSync,
} from "./config.js";
import { NitrogqlConfig, NitrogqlExtension } from "./configFormat.js";
import { loadSchemaJs } from "./loader.js";
import { setMemory } from "./memory.js";

export {
  /**
   * Sets the memory used by the wasm module.
   */
  setMemory,
  initConfigNamespace,
  executeNodeSync,
  executeConfigFileSync,
  loadSchemaJs,
};

export type { NitrogqlConfig, NitrogqlExtension };
