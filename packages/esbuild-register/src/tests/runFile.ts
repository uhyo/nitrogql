import { promisify } from "node:util";
import { execFile } from "node:child_process";
import { pathToFileURL } from "node:url";

const register = new URL("../../dist/index.js", pathToFileURL(__filename));
const hook = new URL("../../dist/hook.mjs", pathToFileURL(__filename));
const nodeVersion = process.versions.node.split(".").map((x) => Number(x)) as [
  number,
  number,
  number
];

export async function runNode(path: string): Promise<string> {
  // >= Node 20.6.0
  if (nodeVersion[0] > 20 || (nodeVersion[0] === 20 && nodeVersion[1] >= 6)) {
    const { stdout } = await promisify(execFile)(
      process.execPath,
      ["--import", register.toString(), path],
      {}
    );
    return stdout;
  } else {
    const { stdout } = await promisify(execFile)(
      process.execPath,
      [
        "--require",
        register.toString(),
        "--experimental-loader",
        hook.toString(),
        path,
      ],
      {}
    );
    return stdout;
  }
}
