import {
  initConfigNamespace,
  executeConfigFileSync,
  executeNodeSync,
} from "./config.js";
import { NitrogqlConfig, NitrogqlExtension } from "./configFormat.js";
import { initContentMapperNamespace } from "./contentMapper.js";
import { loadSchemaJs } from "./loader.js";
import { setMemory } from "./memory.js";

export {
  /**
   * Sets the memory used by the wasm module.
   */
  setMemory,
  initConfigNamespace,
  initContentMapperNamespace,
  executeNodeSync,
  executeConfigFileSync,
  loadSchemaJs,
};

export type { NitrogqlConfig, NitrogqlExtension };
