import { Worker } from "node:worker_threads";
import { once } from "node:events";
import path from "node:path";

export type CommandClient = {
  /**
   * Runs given command as an ES module.
   * Returns the value of default export (without serializing to JSON)
   */
  run: (command: string) => Promise<unknown>;
  close: () => void;
};

export function getCommandClient(): CommandClient {
  const nodeVersion = process.versions.node;
  // @nitrogql/esbuild-register requires different usage
  // depending on whether Node.js supports the `register` API from `node:module`.
  const [major, minor] = nodeVersion.split(".").map((x) => Number(x)) as [
    number,
    number
  ];
  const nodeHasModuleRegisterAPI =
    major > 20 || (major === 20 && minor >= 6) || (major === 18 && minor >= 19);
  const w = new Worker(new URL("./server.js", import.meta.url), {
    env: {
      ...process.env,
      DATA_URL_RESOLUTION_BASE: path.join(process.cwd(), "__entrypoint__"),
    },
    execArgv: nodeHasModuleRegisterAPI
      ? [
          "--no-warnings",
          "--import=@nitrogql/esbuild-register",
          "--inspect-brk",
        ]
      : [
          "--no-warnings",
          "--require=@nitrogql/esbuild-register",
          "--experimental-loader=@nitrogql/esbuild-register/hook",
        ],
  });
  w.on("error", (error) => {
    console.error(error);
  });
  w.on("exit", () => {
    console.error("Worker exited");
  });
  return {
    run: async (command: string) => {
      // console.error("run", command);
      w.postMessage(command);
      const [result] = await once(w, "message");
      await new Promise((resolve) => setTimeout(resolve, 100));
      if (result.error) {
        console.error(result.error);
        throw result.error;
      }
      // console.error("line", result.result);
      return result.result;
    },
    close: () => {
      w.terminate();
    },
  };
}
