#!/usr/bin/env node
// Runs TypeScript 7.1 (installed as `typescript7`), which supports content mappers.
// The `typescript` package is kept at 6.x because Next.js requires its JavaScript API.
import { spawnSync } from "node:child_process";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { argv, execPath, exit } from "node:process";

const require = createRequire(import.meta.url);
const tsc = join(
  dirname(require.resolve("typescript7/package.json")),
  "bin/tsc",
);
const { status } = spawnSync(execPath, [tsc, ...argv.slice(2)], {
  stdio: "inherit",
});
exit(status ?? 1);
