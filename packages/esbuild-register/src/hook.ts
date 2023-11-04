import { LoadHook, ResolveHook } from "node:module";
import { transform } from "esbuild";
import { loadTsConfig } from "./tsconfig.js";
import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import {
  decideOutputFormatOfFile,
  rawSourceToText,
  resolveModule,
} from "./core.js";

const tsExtensions = /\.(?:[cm]?ts|tsx)$/;

export const resolve: ResolveHook = async (
  specifier,
  context,
  defaultResolve
) => {
  const resolved = await resolveModule(specifier, context.parentURL);
  if (resolved === undefined) {
    return defaultResolve(specifier, context);
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
  console.error("tsUrl", tsUrl);
  if (tsUrl !== undefined) {
    const rawSource = await readFile(fileURLToPath(tsUrl), {
      encoding: "utf-8",
    });
    const tsconfig = await loadTsConfig(tsUrl);
    const outputFormat = await decideOutputFormatOfFile(tsUrl);
    const source = await transform(rawSourceToText(rawSource), {
      loader: "ts",
      tsconfigRaw: tsconfig,
      format: outputFormat,
    });
    console.error("source!", outputFormat, source.code);
    return {
      shortCircuit: true,
      format: outputFormat === "cjs" ? "commonjs" : "module",
      source: source.code,
    };
  }
  const loadResult = await nextLoad(url, context);
  return loadResult;
};
