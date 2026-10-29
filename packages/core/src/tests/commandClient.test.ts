import { describe, expect, it } from "vitest";

// Workers run the built server, so the built code is tested.
const dist = new URL("../../dist/", import.meta.url);
const { getCommandClient }: typeof import("../command/commandClient.js") =
  await import(new URL("command/commandClient.js", dist).href);

let commands = 0;
/**
 * Returns a command that counts how many times it has run in its worker.
 * Each worker has its own global object, so the count tells whether commands
 * share a worker. Commands differ, as a worker imports the same command once.
 */
const count = () => `// ${commands++}
globalThis.count = (globalThis.count ?? 0) + 1;
export default globalThis.count;
`;

describe("isolated command client", () => {
  it("runs each command in a new worker outside a session", async () => {
    const client = getCommandClient({ isolated: true });
    try {
      expect(await client.run(count())).toBe(1);
      expect(await client.run(count())).toBe(1);
    } finally {
      client.close();
    }
  });

  it("shares a worker in a session", async () => {
    const client = getCommandClient({ isolated: true });
    try {
      client.beginSession();
      expect(await client.run(count())).toBe(1);
      expect(await client.run(count())).toBe(2);
      client.endSession();
      // The next session starts a new worker.
      client.beginSession();
      expect(await client.run(count())).toBe(1);
      client.endSession();
      expect(await client.run(count())).toBe(1);
    } finally {
      client.close();
    }
  });

  it("keeps the worker until the outermost session ends", async () => {
    const client = getCommandClient({ isolated: true });
    try {
      client.beginSession();
      client.beginSession();
      expect(await client.run(count())).toBe(1);
      client.endSession();
      expect(await client.run(count())).toBe(2);
      client.endSession();
      expect(await client.run(count())).toBe(1);
    } finally {
      client.close();
    }
  });

  it("starts a new worker when the worker of the session exits", async () => {
    const client = getCommandClient({ isolated: true });
    try {
      client.beginSession();
      expect(await client.run(count())).toBe(1);
      await expect(client.run("process.exit(1);")).rejects.toThrow();
      expect(await client.run(count())).toBe(1);
      client.endSession();
    } finally {
      client.close();
    }
  });
});

describe("shared command client", () => {
  it("shares a worker with or without a session", async () => {
    const client = getCommandClient();
    try {
      expect(await client.run(count())).toBe(1);
      client.beginSession();
      expect(await client.run(count())).toBe(2);
      client.endSession();
      expect(await client.run(count())).toBe(3);
    } finally {
      client.close();
    }
  });
});
