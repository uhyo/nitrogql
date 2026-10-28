/**
 * @file Synchronous version of the hooks in `hook.mts`, for `module.registerHooks`.
 *
 * Hooks registered with `module.register` run in a separate thread. Each
 * `require` from a CommonJS module loaded through them waits for a round trip
 * to that thread, which made loading a package such as `graphql` several times
 * slower. These hooks run in the thread that loads modules, and also apply to
 * `require`.
 */

import { readFileSync } from "node:fs";
import { pathToFileURL } from "node:url";
import { transformSync } from "esbuild";
import { loadTsConfigSync } from "./tsconfig.js";
import { decideOutputFormatOfFileSync, resolveModuleSync } from "./core.js";

// `@types/node` does not have the types of `module.registerHooks` yet.

type ResolveContext = {
  conditions: string[];
  importAttributes: Record<string, string>;
  parentURL: string | undefined;
};
type ResolveResult = {
  url: string;
  format?: string | null | undefined;
  importAttributes?: Record<string, string> | undefined;
  shortCircuit?: boolean | undefined;
};
type LoadContext = {
  conditions: string[];
  format: string | null | undefined;
  importAttributes: Record<string, string>;
};
type LoadResult = {
  format: string | null | undefined;
  source?: string | ArrayBuffer | ArrayBufferView | null | undefined;
  shortCircuit?: boolean | undefined;
};

export type SyncHooks = {
  resolve: (
    specifier: string,
    context: ResolveContext,
    nextResolve: (specifier: string, context?: ResolveContext) => ResolveResult,
  ) => ResolveResult;
  load: (
    url: string,
    context: LoadContext,
    nextLoad: (url: string, context?: LoadContext) => LoadResult,
  ) => LoadResult;
};

export type RegisterHooks = (hooks: Partial<SyncHooks>) => unknown;

const tsExtensions = /\.(?:[cm]?ts|tsx)$/;

const isNodeModules = (url: string) => {
  return url.includes("/node_modules/");
};

/**
 * Creates the hooks.
 */
export function createSyncHooks(): SyncHooks {
  /**
   * The base URL to use when resolving relative to data: URL modules.
   * By default, data: URL cannot import relative modules. This option
   * allows you to change this behavior.
   */
  const dataUrlResolutionBaseUrl: string | undefined =
    process.env.DATA_URL_RESOLUTION_BASE &&
    pathToFileURL(process.env.DATA_URL_RESOLUTION_BASE).toString();
  /**
   * Whether to include node_modules to conversion target.
   */
  const includeNodeModules: boolean = !!process.env.INCLUDE_NODE_MODULES;

  return {
    resolve(specifier, context, nextResolve) {
      if (
        context.parentURL?.startsWith("data:") &&
        dataUrlResolutionBaseUrl !== undefined
      ) {
        context = { ...context, parentURL: dataUrlResolutionBaseUrl };
      }
      // for speed, we don't run custom resolver for node_modules
      // unless explicitly requested
      if (
        !includeNodeModules &&
        context.parentURL !== undefined &&
        isNodeModules(context.parentURL)
      ) {
        return nextResolve(specifier, context);
      }
      const resolved = resolveModuleSync(specifier, context.parentURL);
      if (resolved === undefined) {
        return nextResolve(specifier, context);
      }
      return {
        shortCircuit: true,
        url: resolved,
      };
    },
    load(url, context, nextLoad) {
      if (!url.startsWith("file:") || !tsExtensions.test(url)) {
        return nextLoad(url, context);
      }
      const tsUrl = new URL(url);
      const rawSource = readFileSync(tsUrl, { encoding: "utf-8" });
      const tsconfig = loadTsConfigSync(tsUrl);
      const outputFormat = decideOutputFormatOfFileSync(tsUrl);
      const source = transformSync(rawSource, {
        loader: "ts",
        tsconfigRaw: tsconfig?.content,
        format: outputFormat,
      });
      return {
        shortCircuit: true,
        format: outputFormat === "cjs" ? "commonjs" : "module",
        source: source.code,
      };
    },
  };
}
