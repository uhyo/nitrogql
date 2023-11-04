import { access } from "node:fs/promises";
import path from "node:path";
import { pathToFileURL, fileURLToPath } from "node:url";
import {
  loadPackageJson,
  loadPackageJsonSync,
  loadTsConfig,
  loadTsConfigSync,
} from "./tsconfig.js";
import { existsSync } from "node:fs";
import { resolvePaths } from "./paths.js";
import { parseJSONC } from "./parseJSONC.js";

const tsExtensions = /\.(?:[cm]?ts|tsx)$/;

export async function resolveModule(
  specifier: string,
  parentURL: string | undefined
): Promise<string | undefined> {
  if (specifier.startsWith("node:") || specifier.startsWith("data:")) {
    return undefined;
  }
  parentURL ??= pathToFileURL(
    path.join(process.cwd(), "__entrypoint__")
  ).toString();
  let candidates: {
    specifier: string;
    parentURL: string;
  }[];
  const tsConfig = await loadTsConfig(new URL(parentURL));
  if (tsConfig !== undefined) {
    const { url: tsConfigUrl, content } = tsConfig;
    const { baseUrl, paths } = parseJSONC(content)?.compilerOptions ?? {};
    if (paths !== undefined) {
      const resolved = resolvePaths(specifier, paths);
      if (resolved !== undefined) {
        candidates = resolved.map((resolved) => {
          return {
            specifier: resolved,
            parentURL: pathToFileURL(
              path.resolve(
                fileURLToPath(tsConfigUrl),
                "..",
                baseUrl ?? "./__entrypoint__"
              )
            ).toString(),
          };
        });
      }
    }
  }

  try {
    candidates ??= [
      {
        specifier,
        parentURL,
      },
    ];
    for (const { specifier, parentURL } of candidates) {
      if (isExternalSpecifier(specifier)) {
        continue;
      }
      const url = new URL(specifier, parentURL);
      const tsUrl = await mapJsToTs(url);

      if (tsUrl !== undefined) {
        return tsUrl.toString();
      }
      // Allow .ts files
      if (tsExtensions.test(url.pathname)) {
        await access(url);
        return url.toString();
      }
      // Try CommonJS-style resolution
      for (const ext of [".ts", ".tsx", ".cts", ".mts"]) {
        const tsUrl = new URL(url.pathname + ext, url);
        const exists = await access(tsUrl)
          .then(() => true)
          .catch(() => false);
        if (exists) {
          return tsUrl.toString();
        }
      }
    }
  } catch {}
  return undefined;
}

export function resolveModuleSync(
  specifier: string,
  parentURL: string | undefined
): string | undefined {
  if (specifier.startsWith("node:") || specifier.startsWith("data:")) {
    return undefined;
  }

  parentURL ??= pathToFileURL(
    path.join(process.cwd(), "__entrypoint__")
  ).toString();
  let candidates: {
    specifier: string;
    parentURL: string;
  }[];
  const tsConfig = loadTsConfigSync(new URL(parentURL));
  if (tsConfig !== undefined) {
    const { url: tsConfigUrl, content } = tsConfig;
    const { baseUrl, paths } = parseJSONC(content)?.compilerOptions ?? {};
    if (paths !== undefined) {
      const resolved = resolvePaths(specifier, paths);
      if (resolved !== undefined) {
        candidates = resolved.map((resolved) => {
          return {
            specifier: resolved,
            parentURL: pathToFileURL(
              path.resolve(
                fileURLToPath(tsConfigUrl),
                "..",
                baseUrl ?? "./__entrypoint__"
              )
            ).toString(),
          };
        });
      }
    }
  }

  try {
    candidates ??= [
      {
        specifier,
        parentURL,
      },
    ];
    for (const { specifier, parentURL } of candidates) {
      if (isExternalSpecifier(specifier)) {
        continue;
      }
      const url = new URL(specifier, parentURL);
      const tsUrl = mapJsToTsSync(url);

      if (tsUrl !== undefined) {
        return tsUrl.toString();
      }
      // Allow .ts files
      if (tsExtensions.test(url.pathname)) {
        if (existsSync(url)) {
          return url.toString();
        } else {
          return undefined;
        }
      }
      // Try CommonJS-style resolution
      for (const ext of [".ts", ".tsx", ".cts", ".mts"]) {
        const tsUrl = new URL(url.pathname + ext, url);
        if (existsSync(tsUrl)) {
          return tsUrl.toString();
        }
      }
    }
  } catch {}
  return undefined;
}

export async function decideOutputFormatOfFile(
  url: URL
): Promise<"cjs" | "esm"> {
  if (url.pathname.endsWith(".cts")) {
    return "cjs";
  }
  if (url.pathname.endsWith(".mts")) {
    return "esm";
  }

  const packageJson = await loadPackageJson(url);
  if (packageJson) {
    const { content } = packageJson;
    const { type } = JSON.parse(content);
    if (type === "module") {
      return "esm";
    }
    if (type === "commonjs") {
      return "cjs";
    }
  }
  return "cjs";
}

export function decideOutputFormatOfFileSync(url: URL): "cjs" | "esm" {
  if (url.pathname.endsWith(".cts")) {
    return "cjs";
  }
  if (url.pathname.endsWith(".mts")) {
    return "esm";
  }

  const packageJson = loadPackageJsonSync(url);
  if (packageJson) {
    const { content } = packageJson;
    const { type } = JSON.parse(content);
    if (type === "module") {
      return "esm";
    }
    if (type === "commonjs") {
      return "cjs";
    }
  }
  return "cjs";
}

export function rawSourceToText(
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

const jsExtensions = /\.([cm]?js|jsx)$/;
const jsToTs = {
  js: ["ts", "tsx"],
  jsx: ["tsx"],
  cjs: ["cts"],
  mjs: ["mts"],
};

export async function mapJsToTs(url: URL): Promise<URL | undefined> {
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

export function mapJsToTsSync(url: URL): URL | undefined {
  const m = jsExtensions.exec(url.pathname);
  if (m !== null) {
    const matchedExt = m[1] as keyof typeof jsToTs;
    const tsExts = jsToTs[matchedExt];
    for (const ext of tsExts) {
      const tsUrl = new URL(
        url.pathname.slice(0, -matchedExt.length) + ext,
        url
      );
      if (existsSync(tsUrl)) {
        return tsUrl;
      }
    }
  }
  return undefined;
}

function isExternalSpecifier(specifier: string) {
  // has protocol
  if (/^[a-zA-Z]+:/.test(specifier)) {
    return false;
  }
  // relative path
  if (specifier.startsWith("./") || specifier.startsWith("../")) {
    return false;
  }
  return true;
}
