import type { AsyncTransformer, TransformedSource } from "@jest/transform";
import { executeConfigFileSync } from "@nitrogql/core";
import { init } from "@nitrogql/loader-core";
import { readFile } from "node:fs/promises";

const { initiateTask, getLog } = await init();

let lastLoadedConfigPath: string | undefined = undefined;

export type TransformerConfig = {
  /**
   * The path to the nitrogql config file.
   */
  configFile?: string;
};

const transformer: AsyncTransformer<TransformerConfig> = {
  async processAsync(
    sourceText,
    sourcePath,
    options
  ): Promise<TransformedSource> {
    const configFile = options.transformerConfig.configFile;
    const task = initiateTask(sourcePath, sourceText);

    if (lastLoadedConfigPath !== configFile && configFile) {
      const configFileSource = configFileIsJS(configFile)
        ? executeConfigFileSync(configFile)
        : await readFile(configFile, "utf-8");
      task.loadConfig(configFileSource);
    }
    lastLoadedConfigPath = configFile;

    while (true) {
      const status = task.status();
      switch (status.status) {
        case "fileRequired": {
          const requiredFiles = status.files;
          await Promise.all(
            requiredFiles.map(async (requiredFile) => {
              const requiredFileSource = await readFile(requiredFile, "utf-8");
              task.supplyFile(requiredFile, requiredFileSource);
            })
          );
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
