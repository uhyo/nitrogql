import { LoadHook, ResolveHook } from "node:module";
import { transform } from "esbuild";
import { loadPackageJson, loadTsConfig } from "./tsconfig.js";
import { access, readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";

const jsExtensions = /\.([cm]?js|jsx)$/;
const tsExtensions = /\.(?:[cm]?ts|tsx)$/;
const jsToTs = {
  js: ["ts", "tsx"],
  jsx: ["tsx"],
  cjs: ["cts"],
  mjs: ["mts"],
};

export const resolve: ResolveHook = async (
  specifier,
  context,
  defaultResolve
) => {
  if (specifier.startsWith("node:") || specifier.startsWith("data:")) {
    return defaultResolve(specifier, context);
  }
  try {
    const url = new URL(specifier, context.parentURL);
    const tsUrl = await mapJsToTs(url);

    if (tsUrl !== undefined) {
      return {
        shortCircuit: true,
        url: tsUrl.toString(),
      };
    }
    // Allow .ts files
    if (tsExtensions.test(url.pathname)) {
      await access(url);
      console.error("acc!");
      return {
        shortCircuit: true,
        url: url.toString(),
      };
    }
    // Try CommonJS-style resolution
    for (const ext of [".ts", ".tsx", ".cts", ".mts"]) {
      const tsUrl = new URL(url.pathname + ext, url);
      const exists = await access(tsUrl)
        .then(() => true)
        .catch(() => false);
      if (exists) {
        return {
          shortCircuit: true,
          url: tsUrl.toString(),
        };
      }
    }
  } catch {}
  return defaultResolve(specifier, context);
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
    const tsconfig = await loadTsConfig(new URL("../", tsUrl));
    const outputFormat = await decideOutputFormatOfFile(new URL(tsUrl));
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

async function mapJsToTs(url: URL): Promise<URL | undefined> {
  // Map .js to .ts
  const m = jsExtensions.exec(url.pathname);
  if (m !== null) {
    const matchedExt = m[1] as keyof typeof jsToTs;
    const tsExts = jsToTs[matchedExt];
    for (const ext of tsExts) {
      const tsUrl = new URL(
        url.pathname.slice(0, -matchedExt.length) + ext,
        url
      );
      const exists = await access(tsUrl)
        .then(() => true)
        .catch(() => false);
      if (exists) {
        return tsUrl;
      }
    }
  }
  return undefined;
}

function rawSourceToText(
  source: string | ArrayBuffer | ArrayBufferView
): string {
  if (typeof source === "string") {
    return source;
  }
  if (source instanceof ArrayBuffer) {
    return Buffer.from(source).toString("utf8");
  }
  return Buffer.from(
    source.buffer,
    source.byteOffset,
    source.byteLength
  ).toString("utf8");
}

async function decideOutputFormatOfFile(url: URL): Promise<"cjs" | "esm"> {
  if (url.pathname.endsWith(".cts")) {
    return "cjs";
  }
  if (url.pathname.endsWith(".mts")) {
    return "esm";
  }

  const packageJson = await loadPackageJson(new URL("../", url));
  if (packageJson) {
    const { type } = JSON.parse(packageJson);
    if (type === "module") {
      return "esm";
    }
    if (type === "commonjs") {
      return "cjs";
    }
  }
  return "cjs";
}
