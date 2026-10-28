import * as nodeModule from "node:module";

/**
 * Whether to run TypeScript files with Node.js' built-in type stripping
 * instead of @nitrogql/esbuild-register.
 * Set `NITROGQL_DISABLE_TYPE_STRIPPING` to opt out.
 */
export function useNodeTypeStripping(): boolean {
  if (process.env.NITROGQL_DISABLE_TYPE_STRIPPING) {
    return false;
  }
  // `@types/node` does not have the types of these yet.
  const { typescript } = process.features as {
    typescript?: string | false | undefined;
  };
  const { registerHooks } = nodeModule as { registerHooks?: unknown };
  // `module.registerHooks` is needed for commands to import packages.
  // See `registerDataUrlResolution`.
  return !!typescript && registerHooks !== undefined;
}

/**
 * Node.js options needed to run TypeScript files.
 */
export function typeScriptExecArgv(): string[] {
  return useNodeTypeStripping() ? [] : ["--import=@nitrogql/esbuild-register"];
}
