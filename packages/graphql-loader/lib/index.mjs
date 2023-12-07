// @ts-check

import path from "node:path";
import { readFile } from "node:fs/promises";
import { executeConfigFileSync } from "@nitrogql/core";
import { init } from "@nitrogql/loader-core";

const { initiateTask, getLog } = await init();

let lastLoadedConfigPath = undefined;

/**
 * @type {import('webpack').LoaderDefinitionFunction}
 */
export default function graphQLLoader(source) {
  const callback = this.async();
  (async () => {
    const options = this.getOptions();
    const configFile = options?.configFile;
    const configFilePath =
      configFile && path.resolve(this.rootContext, configFile);
    if (configFilePath) {
      this.addDependency(configFilePath);
    }

    const task = initiateTask(this.resourcePath, source);

    if (lastLoadedConfigPath !== configFilePath && configFilePath) {
      const configFileSource = configFileIsJS(configFilePath)
        ? executeConfigFileSync(configFilePath)
        : await readFile(configFilePath, "utf-8");
      task.loadConfig(configFileSource);
    }
    lastLoadedConfigPath = configFilePath;

    // Load all required files
    while (true) {
      const status = task.status();
      switch (status.status) {
        case "fileRequired": {
          const requiredFiles = status.files;
          await Promise.all(
            requiredFiles.map(async (requiredFile) => {
              this.addDependency(requiredFile);
              const requiredFileSource = await readFile(requiredFile, "utf-8");
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
  })()
    .then(
      (res) => callback(null, res),
      (err) => {
        callback(err);
      }
    )
    .finally(() => {
      const log = getLog();
      if (log !== undefined) {
        console.debug(log);
      }
    });
}

function configFileIsJS(configFile) {
  return /\.[cm]?[jt]s$/.test(configFile);
}
