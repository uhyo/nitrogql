import { promisify } from "node:util";
import { execFile } from "node:child_process";
import { pathToFileURL } from "node:url";

const registerMJS = new URL("../../dist/index.mjs", pathToFileURL(__filename));
const registerCJS = new URL("../../dist/index.cjs", pathToFileURL(__filename));
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
      ["--import", registerMJS.toString(), path],
      {}
    );
    return stdout;
  } else {
    const { stdout } = await promisify(execFile)(
      process.execPath,
      [
        "--require",
        registerCJS.toString(),
        "--experimental-loader",
        hook.toString(),
        path,
      ],
      {}
    );
    return stdout;
  }
}
