import readline from "node:readline";
import { stdout, stdin } from "node:process";
import { parentPort } from "node:worker_threads";
import { CommandRunner } from "./commandRunner.js";

const commandRunner = new CommandRunner();
console.error("CommandRunner started");

if (parentPort === null) {
  // standalone server mode
  const rl = readline.createInterface({
    input: stdin,
  });

  rl.on("line", (line) => {
    console.error("Got line:", line);
    const parsed = JSON.parse(line);
    if (typeof parsed !== "string") {
      console.error("Input must be a JSON string");
      return;
    }
    commandRunner.run(parsed);
  });
} else {
  // worker mode
  parentPort.on("message", (message) => {
    console.error("Got message:", message);
    if (typeof message !== "string") {
      console.error("Input must be a string");
      return;
    }
    commandRunner.run(message);
  });
}

for await (const result of commandRunner.output) {
  console.error("Got result:", result);
  if (result.error) {
    console.error(result.error);
    if (parentPort !== null) {
      parentPort.postMessage({
        error: result.error,
      });
    }
  } else {
    if (parentPort !== null) {
      parentPort.postMessage({
        result: result.result,
      });
    } else {
      stdout.write(JSON.stringify(result.result) + "\n");
    }
  }
}
