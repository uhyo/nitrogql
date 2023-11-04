import { loadTsConfigSync } from "./tsconfig.js";
import { decideOutputFormatOfFileSync, rawSourceToText } from "./core.js";
import { transformSync } from "esbuild";

export const cjsHook = (code: string, fileName: string): string => {
  const tsconfig = loadTsConfigSync(new URL("../", fileName));
  const outputFormat = decideOutputFormatOfFileSync(new URL(fileName));
  const source = transformSync(rawSourceToText(code), {
    loader: "ts",
    tsconfigRaw: tsconfig,
    format: outputFormat,
  });
  return source.code;
};
