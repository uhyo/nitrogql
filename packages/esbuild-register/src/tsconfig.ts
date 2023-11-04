import { readFileSync } from "node:fs";
import { readFile } from "node:fs/promises";

export async function loadTsConfig(dirUrl: URL): Promise<string | undefined> {
  return loadConfig(dirUrl, "tsconfig.json");
}

export function loadTsConfigSync(dirUrl: URL): string | undefined {
  return loadConfigSync(dirUrl, "tsconfig.json");
}

export async function loadPackageJson(
  dirUrl: URL
): Promise<string | undefined> {
  return loadConfig(dirUrl, "package.json");
}

export function loadPackageJsonSync(dirUrl: URL): string | undefined {
  return loadConfigSync(dirUrl, "package.json");
}

async function loadConfig(
  dirUrl: URL,
  fileName: string
): Promise<string | undefined> {
  while (true) {
    try {
      return await readFile(new URL(fileName, dirUrl).toString(), {
        encoding: "utf-8",
      });
    } catch (e) {
      if (
        e !== null &&
        typeof e === "object" &&
        "code" in e &&
        e.code === "ENOENT"
      ) {
        const parent = new URL("../", dirUrl);
        if (parent.toString() === dirUrl.toString()) {
          return undefined;
        }
        dirUrl = parent;
      } else {
        throw e;
      }
    }
  }
}

function loadConfigSync(dirUrl: URL, fileName: string): string | undefined {
  while (true) {
    try {
      return readFileSync(new URL(fileName, dirUrl).toString(), {
        encoding: "utf-8",
      });
    } catch (e) {
      if (
        e !== null &&
        typeof e === "object" &&
        "code" in e &&
        e.code === "ENOENT"
      ) {
        const parent = new URL("../", dirUrl);
        if (parent.toString() === dirUrl.toString()) {
          return undefined;
        }
        dirUrl = parent;
      } else {
        throw e;
      }
    }
  }
}
