import { readFileSync } from "node:fs";
import type {
  SyncTransformer,
  TransformedSource,
  TransformerCreator,
} from "@jest/transform";
import { executeConfigFileSync } from "@nitrogql/core";
import { init } from "@nitrogql/loader-core";

export type TransformerConfig = {
  /**
   * The path to the nitrogql config file.
   */
  configFile?: string;
  /**
   * Additional transformer to apply to the generated source code.
   */
  additionalTransformer?:
    | string
    | [transformer: string, transformerConfig: unknown];
  /**
   * Suffix to add to filename when passing code to the additional transformer.
   * @default ".js"
   */
  additionalTransformerFilenameSuffix?: string;
};

const createTransformer: TransformerCreator<
  SyncTransformer<TransformerConfig>,
  TransformerConfig
> = async (options) => {
  const { initiateTask } = await init();

  let lastLoadedConfigPath: string | undefined = undefined;

  const additionalTransformer = options?.additionalTransformer
    ? await loadTransformer(options.additionalTransformer)
    : undefined;

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

      let result: string;
      loop: while (true) {
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
            result = task.emit();
            task.free();
            break loop;
          }
        }
      }
      if (additionalTransformer === undefined) {
        return { code: result };
      }
      const transformed = additionalTransformer[0].process(
        result,
        sourcePath +
          (options.transformerConfig.additionalTransformerFilenameSuffix ??
            ".js"),
        {
          ...options,
          transformerConfig: additionalTransformer[1],
        }
      );
      return transformed;
    },
  };
  return transformer;
};

export default {
  createTransformer,
};

function configFileIsJS(configFile: string) {
  return /\.[cm]?[jt]s$/.test(configFile);
}

async function loadTransformer(
  transformer: string | [transformer: string, transformerConfig: unknown]
): Promise<[transformer: SyncTransformer<unknown>, config: unknown]> {
  let transformerModule, transformerConfig: unknown;
  if (typeof transformer === "string") {
    transformerModule = await import(transformer);
  } else {
    transformerModule = await import(transformer[0]);
    transformerConfig = transformer[1];
  }
  while (transformerModule.default) {
    transformerModule = transformerModule.default;
  }
  if (transformerModule.createTransformer) {
    transformerModule = await transformerModule.createTransformer(
      transformerConfig
    );
  }
  return [transformerModule, transformerConfig];
}
