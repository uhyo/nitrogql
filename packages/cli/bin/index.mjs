#! /usr/bin/env node

import { argv, exit } from "node:process";
import { spawn } from "node:child_process";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const args = argv.slice(2);

const child = spawn(
  "node",
  [
    "--no-warnings",
    "--experimental-wasi-unstable-preview1",
    join(fileURLToPath(import.meta.url), "../main.mjs"),
    ...args,
  ],
  {
    stdio: "inherit",
  }
);

child.on("error", (error) => {
  console.error(`Failed to start subprocess:\n${error}`);
  exit(1);
});

child.on("close", (code) => {
  process.exit(code ?? 1);
});
