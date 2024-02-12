import child_process from "node:child_process";
import * as module from "node:module";
import { Worker } from "node:worker_threads";
import { once } from "node:events";
import path from "node:path";
import readline from "node:readline";
import { fileURLToPath } from "node:url";

export type CommandClient = {
  /**
   * Runs given command as an ES module.
   * Returns the value of default export (without serializing to JSON)
   */
  run: (command: string) => Promise<unknown>;
  close: () => void;
};

export function getCommandClient(): CommandClient {
  // @nitrogql/esbuild-register requires different usage
  // depending on whether Node.js supports the `register` API from `node:module`.
  // @ts-expect-error module.register does not exist yet
  const nodeHasModuleRegisterAPI = !!module.register;
  if (nodeHasModuleRegisterAPI) {
    return getWorkerCommandClient();
  } else {
    return getProcessCommandClient();
  }
}

function getWorkerCommandClient(): CommandClient {
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

function getProcessCommandClient(): CommandClient {
  const childProcess = child_process.spawn(
    process.execPath,
    [
      "--no-warnings",
      "--require=@nitrogql/esbuild-register",
      "--experimental-loader=@nitrogql/esbuild-register/hook",
      fileURLToPath(new URL("./server.js", import.meta.url)),
    ],
    {
      stdio: ["pipe", "pipe", "inherit"],
      env: {
        ...process.env,
        DATA_URL_RESOLUTION_BASE: path.join(process.cwd(), "__entrypoint__"),
      },
    }
  );
  childProcess.on("error", (error) => {
    console.error(error);
  });
  childProcess.on("exit", () => {
    console.error("Child process exited");
  });
  const rl = readline.createInterface({
    input: childProcess.stdout,
    output: childProcess.stdin,
  });
  return {
    run: async (command: string) => {
      const commandString = JSON.stringify(command);
      childProcess.stdin.write(commandString + "\n");
      const [line] = await once(rl, "line");
      const result = JSON.parse(line);
      if (result.error) {
        throw result.error;
      }
      return result.result;
    },
    close: () => {
      childProcess.kill();
    },
  };
}
