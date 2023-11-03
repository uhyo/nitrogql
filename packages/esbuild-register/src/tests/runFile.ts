import { promisify } from "node:util";
import { execFile } from "node:child_process";

const register = new URL("../../dist/index.js", import.meta.url);

export async function runNode(path: string): Promise<string> {
  const { stdout } = await promisify(execFile)(
    process.execPath,
    ["--import", register.toString(), path],
    {}
  );
  return stdout;
}
