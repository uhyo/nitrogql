import inspector from "node:inspector";
import { parentPort } from "node:worker_threads";
import { CommandRunner } from "./commandRunner.js";

const commandRunner = new CommandRunner();
// console.error("CommandRunner started", process.version);

if (parentPort === null) {
  throw new Error("This module must be run as a worker thread");
}

// https://github.com/nodejs/node/issues/26609
if (process.execArgv.includes("--inspect-brk")) {
  inspector.open();
  inspector.waitForDebugger();
  debugger;
}

parentPort.on("message", (message) => {
  // console.error("Got message:", message);
  if (typeof message !== "string") {
    console.error("Input must be a string");
    return;
  }
  commandRunner.run(message);
});

for await (const result of commandRunner.output) {
  // console.error("Got result:", result);
  if (result.error) {
    console.error(result.error);
    parentPort.postMessage({
      error: result.error,
    });
  } else {
    parentPort.postMessage({
      result: result.result,
    });
  }
}
