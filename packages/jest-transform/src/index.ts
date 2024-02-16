import { readFileSync } from "node:fs";
import type { SyncTransformer, TransformedSource } from "@jest/transform";
import { executeConfigFileSync } from "@nitrogql/core";
import { init } from "@nitrogql/loader-core";

const { initiateTask, getLog } = await init();

let lastLoadedConfigPath: string | undefined = undefined;

export type TransformerConfig = {
  /**
   * The path to the nitrogql config file.
   */
  configFile?: string;
};

const transformer: SyncTransformer<TransformerConfig> = {
  process(sourceText, sourcePath, options): TransformedSource {
    const configFile = options.transformerConfig.configFile;
    const task = initiateTask(sourcePath, sourceText);

    if (lastLoadedConfigPath !== configFile && configFile) {
      const configFileSource = configFileIsJS(configFile)
        ? executeConfigFileSync(configFile)
        : readFileSync(configFile, "utf-8");
      task.loadConfig(configFileSource);
    }
    lastLoadedConfigPath = configFile;

    while (true) {
      const status = task.status();
      switch (status.status) {
        case "fileRequired": {
          const requiredFiles = status.files;
          for (const requiredFile of requiredFiles) {
            const requiredFileSource = readFileSync(requiredFile, "utf-8");
            task.supplyFile(requiredFile, requiredFileSource);
          }
          break;
        }
        case "ready": {
          const result = task.emit();
          task.free();
          return {
            code: result,
          };
        }
      }
    }
  },
};

export default transformer;

function configFileIsJS(configFile: string) {
  return /\.[cm]?[jt]s$/.test(configFile);
}
