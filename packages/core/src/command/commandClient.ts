import { Worker } from "node:worker_threads";
import readline from "node:readline";
import { once } from "node:events";

export type CommandClient = {
  run: (command: string) => Promise<string>;
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
    execArgv: nodeHasModuleRegisterAPI
      ? []
      : // ? ["--no-warnings", "--import=@nitrogql/esbuild-register"]
        [
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
      console.error("run", command);
      w.postMessage(command);
      const [result] = await once(w, "message");
      if (result.error) {
        console.error(result.error);
        throw result.error;
      }
      console.error("line", result.result);
      return result.result;
    },
    close: () => {
      w.terminate();
    },
  };
}
