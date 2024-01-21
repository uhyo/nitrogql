import readline from "node:readline";
import { stdout, stdin } from "node:process";
import { CommandRunner } from "./commandRunner.js";

const commandRunner = new CommandRunner();
// console.error("CommandRunner started");

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

for await (const result of commandRunner.output) {
  // console.error("Got result:", result);
  if (result.error) {
    console.error(result.error);
  } else {
    process.stdout.write(JSON.stringify(result.result) + "\n");
  }
}
