import readline from "node:readline";
import { stdout, stdin } from "node:process";
import { CommandRunner } from "./commandRunner.js";

const commandRunner = new CommandRunner();

const rl = readline.createInterface({
  input: stdin,
  output: stdout,
});

rl.on("line", (line) => {
  commandRunner.run(line);
});

for await (const result of commandRunner.output) {
  if (result.error) {
    console.error(result.error);
  } else {
    process.stdout.write(JSON.stringify(result.result) + "\n");
  }
}
