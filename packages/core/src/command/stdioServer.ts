import readline from "node:readline";
import { exit, stdin, stdout } from "node:process";
import { getCommandClient } from "./commandClient.js";

/**
 * Command server for the native (non-wasm) build of nitrogql.
 * Reads messages from stdin, one JSON value per line:
 * - A string is a command. Its result is written to stdout as one line of JSON.
 * - `{"session": "begin"}` and `{"session": "end"}` begin and end a session.
 *   They are not responded to.
 * Commands run in command workers, so this decides between Node.js' type
 * stripping and @nitrogql/esbuild-register just like the wasm build does.
 * Each command runs in a new worker unless it is in a session; nitrogql
 * decides which commands share a worker.
 */

const client = getCommandClient({ isolated: true });
const rl = readline.createInterface({
  input: stdin,
});

for await (const line of rl) {
  const message: unknown = JSON.parse(line);
  if (typeof message === "object" && message !== null && "session" in message) {
    if (message.session === "begin") {
      client.beginSession();
    } else if (message.session === "end") {
      client.endSession();
    } else {
      console.error("Unknown session message");
    }
    continue;
  }
  if (typeof message !== "string") {
    console.error("Input must be a JSON string");
    continue;
  }
  try {
    const result = await client.run(message);
    stdout.write(JSON.stringify({ result: result ?? null }) + "\n");
  } catch (error) {
    stdout.write(JSON.stringify({ error: String(error) }) + "\n");
  }
}

// stdin is closed when nitrogql exits.
// Exiting directly avoids waiting for workers to terminate.
exit(0);
