import readline from "node:readline";
import { exit, stdin, stdout } from "node:process";
import { getCommandClient } from "./commandClient.js";

/**
 * Command server for the native (non-wasm) build of nitrogql.
 * Reads commands from stdin, one JSON string per line, and writes the result
 * of each command to stdout as one line of JSON.
 * Commands run in a command worker, so this decides between Node.js' type
 * stripping and @nitrogql/esbuild-register just like the wasm build does.
 */

const client = getCommandClient();
const rl = readline.createInterface({
  input: stdin,
});

for await (const line of rl) {
  const command: unknown = JSON.parse(line);
  if (typeof command !== "string") {
    console.error("Input must be a JSON string");
    continue;
  }
  try {
    const result = await client.run(command);
    stdout.write(JSON.stringify({ result: result ?? null }) + "\n");
  } catch (error) {
    stdout.write(JSON.stringify({ error: String(error) }) + "\n");
  }
}

// stdin is closed when nitrogql exits.
// Exiting directly avoids the worker's "Worker exited" message.
exit(0);
