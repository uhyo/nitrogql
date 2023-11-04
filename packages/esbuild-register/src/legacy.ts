import { pathToFileURL } from "node:url";
import { loadTsConfigSync } from "./tsconfig.js";
import { decideOutputFormatOfFileSync, rawSourceToText } from "./core.js";
import { transformSync } from "esbuild";

export const cjsHook = (code: string, fileName: string): string => {
  const url = pathToFileURL(fileName);
  const tsconfig = loadTsConfigSync(url);
  const outputFormat = decideOutputFormatOfFileSync(url);
  const source = transformSync(rawSourceToText(code), {
    loader: "ts",
    tsconfigRaw: tsconfig?.content,
    format: outputFormat,
  });
  return source.code;
};
