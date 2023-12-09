// @ts-check

import path from "node:path";
import { readFile } from "node:fs/promises";
import { executeConfigFileSync } from "@nitrogql/core";
import { init } from "@nitrogql/loader-core";
import { createFilter } from "@rollup/pluginutils";

const { initiateTask, getLog } = await init();

let lastLoadedConfigPath = undefined;

/**
 * @type {import('rollup').PluginImpl<{
 *   configFile?: string;
 *   include?: string | readonly string[];
 *   exclude?: string | readonly string[];
 * }>}
 */
export default function NitrogqlRollupPlugin(options) {
  const { configFile, include, exclude } = options ?? {};
  const filter = createFilter(include, exclude);
  let root = process.cwd();

  return {
    name: "@nitrogql/rollup-plugin",
    // Vite-specific hook to resolve the root directory of the project.
    // @ts-expect-error
    configResolved(config) {
      root = config.root;
    },
    async transform(source, id) {
      if (!filter(id)) {
        return;
      }

      const configFilePath = configFile && path.resolve(root, configFile);
      if (configFilePath) {
        this.addWatchFile(configFilePath);
      }

      const task = initiateTask(id, source);

      if (lastLoadedConfigPath !== configFilePath && configFilePath) {
        const configFileSource = configFileIsJS(configFilePath)
          ? executeConfigFileSync(configFilePath)
          : await readFile(configFilePath, "utf-8");
        task.loadConfig(configFileSource);
      }
      lastLoadedConfigPath = configFilePath;

      while (true) {
        const status = task.status();
        switch (status.status) {
          case "fileRequired": {
            const requiredFiles = status.files;
            await Promise.all(
              requiredFiles.map(async (requiredFile) => {
                this.addWatchFile(requiredFile);
                const requiredFileSource = await readFile(
                  requiredFile,
                  "utf-8"
                );
                task.supplyFile(requiredFile, requiredFileSource);
              })
            );
            break;
          }
          case "ready": {
            const result = task.emit();
            task.free();
            return result;
          }
        }
      }
    },
  };
}

function configFileIsJS(configFile) {
  return /\.[cm]?[jt]s$/.test(configFile);
}
