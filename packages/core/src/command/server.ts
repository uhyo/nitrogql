import readline from "node:readline";
import { stdout, stdin } from "node:process";
import { workerData } from "node:worker_threads";
import { CommandRunner } from "./commandRunner.js";

const commandRunner = new CommandRunner();
console.error("CommandRunner started");
let signalBuffer: Int32Array | undefined;
if (workerData?.signalBuffer instanceof SharedArrayBuffer) {
  // in a worker thread, we use the signal buffer to communicate
  // with the parent thread.
  signalBuffer = new Int32Array(workerData.signalBuffer);
}

const rl = readline.createInterface({
  input: stdin,
  output: stdout,
});

rl.on("line", (line) => {
  // console.error("Got line:", line);
  const parsed = JSON.parse(line);
  if (typeof parsed !== "string") {
    console.error("Input must be a JSON string");
    return;
  }
  commandRunner.run(parsed);
});

let resultId = 0;

for await (const result of commandRunner.output) {
  // console.error("Got result:", result);
  if (result.error) {
    console.error(result.error);
  } else {
    process.stdout.write(JSON.stringify(result.result) + "\n");
  }
  resultId++;
  if (signalBuffer) {
    Atomics.store(signalBuffer, 0, resultId);
    Atomics.notify(signalBuffer, 0);
  }
}
