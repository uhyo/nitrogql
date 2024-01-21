import { readFileSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { parentDir } from "./util.js";

export async function loadTsConfig(
  targetUrl: URL
): Promise<LoadConfigResult | undefined> {
  return loadConfig(targetUrl, "tsconfig.json");
}

export function loadTsConfigSync(targetUrl: URL): LoadConfigResult | undefined {
  return loadConfigSync(targetUrl, "tsconfig.json");
}

export async function loadPackageJson(
  targetUrl: URL
): Promise<LoadConfigResult | undefined> {
  return loadConfig(targetUrl, "package.json");
}

export function loadPackageJsonSync(
  targetUrl: URL
): LoadConfigResult | undefined {
  return loadConfigSync(targetUrl, "package.json");
}

type LoadConfigResult = {
  url: string;
  content: string;
};

/**
 * Loads config file from the same directory or parent directories of the target.
 */
async function loadConfig(
  targetUrl: URL,
  fileName: string
): Promise<LoadConfigResult | undefined> {
  if (targetUrl.protocol === "data:") {
    // data URL doesn't belong to a file,
    // so we can't load config from the file system.
    return undefined;
  }
  while (true) {
    try {
      const loadUrl = new URL(fileName, targetUrl);
      const content = await readFile(loadUrl, { encoding: "utf-8" });
      return {
        url: loadUrl.toString(),
        content,
      };
    } catch (e) {
      if (
        e !== null &&
        typeof e === "object" &&
        "code" in e &&
        e.code === "ENOENT"
      ) {
        const parent = parentDir(targetUrl);
        if (parent.toString() === targetUrl.toString()) {
          return undefined;
        }
        targetUrl = parent;
      } else {
        throw e;
      }
    }
  }
}

function loadConfigSync(
  targetUrl: URL,
  fileName: string
): LoadConfigResult | undefined {
  while (true) {
    try {
      const loadUrl = new URL(fileName, targetUrl);
      const content = readFileSync(loadUrl, { encoding: "utf-8" });
      return {
        url: loadUrl.toString(),
        content,
      };
    } catch (e) {
      if (
        e !== null &&
        typeof e === "object" &&
        "code" in e &&
        e.code === "ENOENT"
      ) {
        const parent = new URL("../", targetUrl);
        if (parent.toString() === targetUrl.toString()) {
          return undefined;
        }
        targetUrl = parent;
      } else {
        throw e;
      }
    }
  }
}
