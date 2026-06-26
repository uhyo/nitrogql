import { promisify } from "node:util";
import { execFile } from "node:child_process";

const registerMJS = new URL("../../dist/index.mjs", import.meta.url);

export async function runNode(
  path: string,
  options?: {
    dataUrlResolutionBase?: string;
  },
): Promise<string> {
  const env =
    options?.dataUrlResolutionBase !== undefined
      ? { DATA_URL_RESOLUTION_BASE: options.dataUrlResolutionBase }
      : undefined;
  const { stdout } = await promisify(execFile)(
    process.execPath,
    ["--import", registerMJS.toString(), path],
    {
      env,
    },
  );
  return stdout;
}
