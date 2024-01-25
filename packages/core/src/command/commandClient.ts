import { Worker } from "node:worker_threads";

export type CommandClient = {
  run: (command: string) => string;
  close: () => void;
};

export function getCommandClient() {
  const nodeVersion = process.versions.node;
  // @nitrogql/esbuild-register requires different usage
  // depending on whether Node.js supports the `register` API from `node:module`.
  const [major, minor] = nodeVersion.split(".").map((x) => Number(x)) as [
    number,
    number
  ];
  const nodeHasModuleRegisterAPI =
    major > 20 || (major === 20 && minor >= 6) || (major === 18 && minor >= 19);
  const signalBuffer = new SharedArrayBuffer(4);
  const w = new Worker(new URL("./server.js", import.meta.url), {
    execArgv: nodeHasModuleRegisterAPI
      ? [
          "--no-warnings",
          "--import=@nitrogql/esbuild-register",
          "--input-type=module",
        ]
      : [
          "--no-warnings",
          "--require=@nitrogql/esbuild-register",
          "--experimental-loader=@nitrogql/esbuild-register/hook",
          "--input-type=module",
        ],
    stdin: true,
    stdout: true,
    workerData: {
      signalBuffer,
    },
  });
  return {};
  return w;
}
