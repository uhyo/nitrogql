import { register, Module } from "node:module";
import { resolveModuleSync } from "./core.js";
import { fileURLToPath, pathToFileURL } from "node:url";

// ESM loader
const hookUrl = new URL("hook.js", import.meta.url);
register(hookUrl);

// Node.js' loader (>= Node 20.6.0) still depends on Module._resolveFilename,
// so we need to patch it as well.
{
  // @ts-expect-error
  const originalResolveFilename = Module._resolveFilename;
  // @ts-expect-error
  Module._resolveFilename = (specifier: string, parent: Module) => {
    const resolved = resolveModuleSync(
      specifier,
      pathToFileURL(parent.filename).toString()
    );
    if (resolved !== undefined) {
      return fileURLToPath(resolved);
    }
    return originalResolveFilename(specifier, parent);
  };
}

// CJS stuff
// addHook(cjsHook, {
//   extensions: [".js", ".jsx", ".cjs", ".mjs", ".ts", ".tsx", ".cts", ".mts"],
// });
