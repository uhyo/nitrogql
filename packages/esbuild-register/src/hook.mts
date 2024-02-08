import type { InitializeHook, LoadHook, ResolveHook } from "node:module";
import * as module from "node:module";
import { transform } from "esbuild";
import { loadTsConfig } from "./tsconfig.js";
import { readFile } from "node:fs/promises";
import { fileURLToPath, pathToFileURL } from "node:url";
import {
  decideOutputFormatOfFile,
  rawSourceToText,
  resolveModule,
} from "./core.js";

// >= Node 20.6.0
const esmLoaderHasCjsSupport =
  // @ts-expect-error
  module.register !== undefined;

const tsExtensions = /\.(?:[cm]?ts|tsx)$/;

/**
 * The base URL to use when resolving relative to data: URL modules.
 * By default, data: URL cannot import relative modules. This option
 * allows you to change this behavior.
 */
let dataUrlResolutionBaseUrl: string | undefined =
  process.env.DATA_URL_RESOLUTION_BASE &&
  pathToFileURL(process.env.DATA_URL_RESOLUTION_BASE).toString();

/**
 * Whether to include node_modules to conversion target.
 */
let includeNodeModules: boolean = !!process.env.INCLUDE_NODE_MODULES;

export const initialize: InitializeHook<
  | {
      dataUrlResolutionBase?: string;
      includeNodeModules?: boolean;
    }
  | undefined
> = (data) => {
  if (data?.dataUrlResolutionBase !== undefined) {
    dataUrlResolutionBaseUrl = pathToFileURL(
      data.dataUrlResolutionBase
    ).toString();
  }
  if (data?.includeNodeModules !== undefined) {
    includeNodeModules = data.includeNodeModules;
  }
};

const isNodeModules = (url: string) => {
  return url.includes("/node_modules/");
};

export const resolve: ResolveHook = async (
  specifier,
  context,
  defaultResolve
) => {
  if (
    context.parentURL?.startsWith("data:") &&
    dataUrlResolutionBaseUrl !== undefined
  ) {
    context.parentURL = dataUrlResolutionBaseUrl;
  }
  // for speed, we don't run custom resolver for node_modules
  // unless explicitly requested
  if (
    !includeNodeModules &&
    context.parentURL !== undefined &&
    isNodeModules(context.parentURL)
  ) {
    const res = await defaultResolve(specifier, context);
    return res;
  }

  const resolved = await resolveModule(specifier, context.parentURL);
  if (resolved === undefined) {
    const res = await defaultResolve(specifier, context);
    return res;
  }
  return {
    shortCircuit: true,
    url: resolved,
  };
};

export const load: LoadHook = async (url, context, nextLoad) => {
  if (url.startsWith("node:") || url.startsWith("data:")) {
    return nextLoad(url, context);
  }
  // const tsUrl = tsExtensions.test(url)
  //   ? new URL(url)
  //   : await mapJsToTs(new URL(url));
  const tsUrl = tsExtensions.test(url) ? new URL(url) : undefined;
  if (tsUrl !== undefined) {
    const rawSource = await readFile(fileURLToPath(tsUrl), {
      encoding: "utf-8",
    });
    const tsconfig = await loadTsConfig(tsUrl);
    const outputFormat = await decideOutputFormatOfFile(tsUrl);
    if (outputFormat === "cjs" && !esmLoaderHasCjsSupport) {
      // Node.js < 20.6.0 doesn't support returning source when
      // the format is "cjs", so we save work by short-circuiting
      // here.
      return {
        shortCircuit: true,
        format: "commonjs",
      };
    }
    const source = await transform(rawSourceToText(rawSource), {
      loader: "ts",
      tsconfigRaw: tsconfig?.content,
      format: outputFormat,
    });
    return {
      shortCircuit: true,
      format: outputFormat === "cjs" ? "commonjs" : "module",
      source: source.code,
    };
  }
  const loadResult = await nextLoad(url, context);
  // To avoid https://github.com/nodejs/node/issues/50435,
  // we always fill the CommonJS source
  if (loadResult.format === "commonjs") {
    loadResult.source ??= await readFile(
      // @ts-expect-error
      new URL(loadResult.responseURL ?? url),
      "utf-8"
    );
  }
  // console.log((performance.now() - startTime).toFixed(1), "load1", loadResult);
  return loadResult;
};
