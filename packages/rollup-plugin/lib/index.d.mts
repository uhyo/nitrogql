import type { PluginImpl } from "rollup";
declare const nitrogqlRollupPlugin: PluginImpl<{
  /**
   * Path to nitrogql's config file (config.graphql.yaml).
   */
  configFile?: string;
  /**
   * Glob to include files to process.
   */
  include?: string | readonly string[];
  /**
   * Glob to exclude files to process.
   */
  exclude?: string | readonly string[];
}>;

export default nitrogqlRollupPlugin;
