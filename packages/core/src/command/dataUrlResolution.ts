import * as nodeModule from "node:module";
import { pathToFileURL } from "node:url";

// `@types/node` does not have the types of `module.registerHooks` yet.

type ResolveContext = {
  conditions: string[];
  importAttributes: Record<string, string>;
  parentURL: string | undefined;
};
type ResolveResult = {
  url: string;
  format?: string | null | undefined;
  shortCircuit?: boolean | undefined;
};
type RegisterHooks = (hooks: {
  resolve: (
    specifier: string,
    context: ResolveContext,
    nextResolve: (specifier: string, context?: ResolveContext) => ResolveResult,
  ) => ResolveResult;
}) => unknown;

/**
 * Lets commands, which are run as data: URL modules, import packages.
 * Node.js cannot resolve imports from data: URL modules except for absolute
 * URLs and built-in modules, so they are resolved relative to
 * `DATA_URL_RESOLUTION_BASE` instead.
 * @nitrogql/esbuild-register does the same when it is used.
 */
export function registerDataUrlResolution(): void {
  const base = process.env.DATA_URL_RESOLUTION_BASE;
  const { registerHooks } = nodeModule as {
    registerHooks?: RegisterHooks | undefined;
  };
  if (base === undefined || registerHooks === undefined) {
    return;
  }
  const baseUrl = pathToFileURL(base).toString();
  registerHooks({
    resolve(specifier, context, nextResolve) {
      if (context.parentURL?.startsWith("data:")) {
        return nextResolve(specifier, { ...context, parentURL: baseUrl });
      }
      return nextResolve(specifier, context);
    },
  });
}
