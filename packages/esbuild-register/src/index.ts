import * as module from "node:module";
import { addHook } from "pirates";
import { resolveModuleSync } from "./core.js";
import { fileURLToPath, pathToFileURL } from "node:url";
import { cjsHook } from "./legacy.js";

// ESM loader (>= Node 20.6.0)
// @ts-expect-error
if (module.register) {
  const hookUrl = new URL("hook.js", import.meta.url);
  // @ts-expect-error
  module.register(hookUrl);
}

// Node.js' loader (>= Node 20.6.0) still depends on Module._resolveFilename,
// so we need to patch it as well.
{
  const { Module } = module;
  // @ts-expect-error
  const originalResolveFilename = Module._resolveFilename;
  // @ts-expect-error
  Module._resolveFilename = (
    specifier: string,
    parent: NodeJS.Module | undefined
  ) => {
    console.error("_resolveFilename", specifier, parent);
    const resolved = resolveModuleSync(
      specifier,
      parent && pathToFileURL(parent.filename).toString()
    );
    if (resolved !== undefined) {
      return fileURLToPath(resolved);
    }
    return originalResolveFilename(specifier, parent);
  };
}

// CJS stuff
addHook(cjsHook, {
  extensions: [".js", ".jsx", ".cjs", ".mjs", ".ts", ".tsx", ".cts", ".mts"],
});
