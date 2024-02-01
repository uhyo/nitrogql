import * as nodeModule from "node:module";
import { addHook } from "pirates";
import { resolveModuleSync } from "./core.js";
import { fileURLToPath, pathToFileURL } from "node:url";
import { cjsHook } from "./legacy.js";

const isNodeModules = (url: string) => {
  return url.includes("/node_modules/");
};

/**
 * Registers the esbuild hook.
 */
export function register() {
  /**
   * Whether to include node_modules to conversion target.
   */
  const includeNodeModules: boolean = !!process.env.INCLUDE_NODE_MODULES;

  // ESM loader (>= Node 20.6.0 or >= 18.19.0)
  // @ts-expect-error
  if (nodeModule.register) {
    const hookUrl = new URL("hook.mjs", pathToFileURL(__filename));
    // @ts-expect-error
    nodeModule.register(hookUrl);
  }

  // Node.js' loader (>= Node 20.6.0 or >= Node 18.19.0) still depends on Module._resolveFilename,
  // so we need to patch it as well.
  {
    const { Module } = nodeModule;
    // @ts-expect-error
    const originalResolveFilename = Module._resolveFilename;
    // @ts-expect-error
    Module._resolveFilename = (
      specifier: string,
      parent: NodeJS.Module | undefined
    ) => {
      const parentURL = parent && pathToFileURL(parent.filename).toString();
      if (
        !includeNodeModules &&
        parentURL !== undefined &&
        isNodeModules(parentURL)
      ) {
        return originalResolveFilename(specifier, parent);
      }

      const resolved = resolveModuleSync(specifier, parentURL);
      if (resolved !== undefined) {
        return fileURLToPath(resolved);
      }
      return originalResolveFilename(specifier, parent);
    };
  }

  // CJS stuff.
  // Only needed for Node.js < 20.6.0 (as long as the entrypoint is ESM)
  addHook(cjsHook, {
    extensions: [".js", ".jsx", ".cjs", ".mjs", ".ts", ".tsx", ".cts", ".mts"],
  });
}
