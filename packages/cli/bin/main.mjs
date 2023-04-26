import { WASI } from "node:wasi";
import { argv, env, cwd, stdout } from "node:process";
import { readFile } from "node:fs/promises";
import { shim } from "./shim.mjs";
import { resolve } from "node:path";
import * as core from "@nitrogql/core";

const CWD = cwd();
const NITROGQL_FS_SCOPE = env.NITROGQL_FS_SCOPE
  ? resolve(CWD, env.NITROGQL_FS_SCOPE)
  : CWD;
const isTTY = stdout.isTTY;

const wasi = new WASI({
  args: argv.slice(1),
  env: {
    // env_logger
    RUST_LOG_STYLE: isTTY ? "always" : "auto",
    // colored
    CLICOLOR_FORCE: isTTY ? "1" : "0",
    ...env,
    CWD,
  },
  preopens: {
    [NITROGQL_FS_SCOPE]: NITROGQL_FS_SCOPE,
  },
});

let memoryRef = { memory: null };
const importObject = {
  wasi_snapshot_preview1: {
    ...wasi.wasiImport,
    ...shim(wasi.wasiImport, memoryRef, NITROGQL_FS_SCOPE),
  },
  "nitrogql_helper/config": core.config,
};

const wasm = await WebAssembly.compile(
  await readFile(new URL("../wasm/nitrogql-cli.wasm", import.meta.url))
);
const instance = await WebAssembly.instantiate(wasm, importObject);
memoryRef.memory = instance.exports.memory;
core.setMemory(instance.exports.memory);

wasi.start(instance);
