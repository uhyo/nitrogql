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
  const w = new Worker(new URL("./server.js", import.meta.url), {
    env: {
      ...process.env,
      DATA_URL_RESOLUTION_BASE: path.join(process.cwd(), "__entrypoint__"),
    },
    execArgv: ["--no-warnings", "--import=@nitrogql/esbuild-register"],
  });
  w.on("error", (error) => {
    console.error(error);
  });
  w.on("exit", () => {
    console.error("Worker exited");
  });
  return {
    run: async (command: string) => {
      w.postMessage(command);
      const [result] = await once(w, "message");
      if (result.error) {
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
