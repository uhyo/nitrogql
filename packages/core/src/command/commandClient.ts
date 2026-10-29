import { Worker } from "node:worker_threads";
import { once } from "node:events";
import path from "node:path";
import { typeScriptExecArgv } from "../typeStripping.js";

export type CommandClient = {
  /**
   * Runs given command as an ES module.
   * Returns the value of default export (without serializing to JSON)
   */
  run: (command: string) => Promise<unknown>;
  /**
   * Starts a session. Commands in a session share one worker, even in
   * isolated mode, so that modules imported by one command are not loaded
   * again by the next.
   * Sessions can be nested; the session lasts until the outermost one ends.
   */
  beginSession: () => void;
  /**
   * Ends the session started by `beginSession`.
   */
  endSession: () => void;
  close: () => void;
};

export type CommandClientOptions = {
  /**
   * Whether to run each command in a new worker, unless it is in a session.
   * A worker caches the modules it imports, so a shared worker returns stale
   * results after a config or schema file is changed. Long-running processes
   * such as the content mapper need this, while one-shot commands save the
   * startup time of workers by sharing one.
   * The content mapper loads a project in a session, so that the files
   * of a project are loaded by one worker, and loaded again on reopen.
   */
  isolated?: boolean;
};

export function getCommandClient({
  isolated = false,
}: CommandClientOptions = {}): CommandClient {
  if (!isolated) {
    const w = startWorker();
    w.on("error", (error) => {
      console.error(error);
    });
    w.on("exit", () => {
      console.error("Worker exited");
    });
    return {
      run: (command: string) => runCommand(w, command),
      // Every command shares the worker anyway.
      beginSession: () => {},
      endSession: () => {},
      close: () => {
        w.terminate();
      },
    };
  }
  const workers = new Set<Worker>();
  let sessionDepth = 0;
  /**
   * Worker of the current session, started by the first command in it.
   */
  let sessionWorker: Worker | undefined;
  const startIsolatedWorker = () => {
    const w = startWorker();
    workers.add(w);
    w.on("error", (error) => {
      console.error(error);
    });
    w.once("exit", () => {
      workers.delete(w);
      // A later command in the session starts a new one.
      if (sessionWorker === w) {
        sessionWorker = undefined;
      }
    });
    return w;
  };
  return {
    run: async (command: string) => {
      if (sessionDepth > 0) {
        sessionWorker ??= startIsolatedWorker();
        return runCommand(sessionWorker, command);
      }
      const w = startIsolatedWorker();
      try {
        return await runCommand(w, command);
      } finally {
        w.terminate();
      }
    },
    beginSession: () => {
      sessionDepth++;
    },
    endSession: () => {
      // Throwing would abort the wasm module that calls this.
      if (sessionDepth === 0) {
        return;
      }
      sessionDepth--;
      if (sessionDepth === 0) {
        sessionWorker?.terminate();
        sessionWorker = undefined;
      }
    },
    close: () => {
      for (const w of workers) {
        w.terminate();
      }
    },
  };
}

function startWorker(): Worker {
  return new Worker(new URL("./server.js", import.meta.url), {
    env: {
      ...process.env,
      DATA_URL_RESOLUTION_BASE: path.join(process.cwd(), "__entrypoint__"),
    },
    execArgv: ["--no-warnings", ...typeScriptExecArgv()],
  });
}

async function runCommand(w: Worker, command: string): Promise<unknown> {
  // Without this, a command would wait forever for a worker that has exited.
  const exited = new AbortController();
  const onExit = (code: number) => {
    exited.abort(new Error(`Worker exited with code ${code}`));
  };
  w.once("exit", onExit);
  try {
    w.postMessage(command);
    const [result] = await once(w, "message", { signal: exited.signal });
    if (result.error) {
      throw result.error;
    }
    return result.result;
  } finally {
    w.off("exit", onExit);
  }
}
