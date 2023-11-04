import { readFileSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { parentDir } from "./util.js";

export async function loadTsConfig(
  targetUrl: URL
): Promise<string | undefined> {
  return loadConfig(targetUrl, "tsconfig.json");
}

export function loadTsConfigSync(targetUrl: URL): string | undefined {
  return loadConfigSync(targetUrl, "tsconfig.json");
}

export async function loadPackageJson(
  targetUrl: URL
): Promise<string | undefined> {
  return loadConfig(targetUrl, "package.json");
}

export function loadPackageJsonSync(targetUrl: URL): string | undefined {
  return loadConfigSync(targetUrl, "package.json");
}

/**
 * Loads config file from the same directory or parent directories of the target.
 */
async function loadConfig(
  targetUrl: URL,
  fileName: string
): Promise<string | undefined> {
  while (true) {
    try {
      return await readFile(new URL(fileName, targetUrl), {
        encoding: "utf-8",
      });
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

function loadConfigSync(targetUrl: URL, fileName: string): string | undefined {
  while (true) {
    try {
      return readFileSync(new URL(fileName, targetUrl).toString(), {
        encoding: "utf-8",
      });
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
