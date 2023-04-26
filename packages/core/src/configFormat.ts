export type NitrogqlConfig = {
  /**
   * List of paths to schema files.
   */
  schema: string | readonly string[];
  /**
   * List of paths to operation files.
   */
  documents?: string | readonly string[] | undefined;
  extensions?:
    | (Record<string, unknown> & {
        nitrogql?:
          | {
              /**
               * Config related to the 'generate' command.
               */
              generate?:
                | {
                    /**
                     * Mode of output
                     * @default "with-loader-ts-5.0"
                     */
                    mode?:
                      | "with-loader-ts-5.0"
                      | "with-loader-ts-4.0"
                      | "standalone-ts-4.0"
                      | undefined;
                    /**
                     * Path to the output schema type definition file.
                     * Required when using the `generate` command.
                     */
                    schemaOutput?: string | undefined;
                    /**
                     * Module specifier for importing schema types from operations.
                     * Defaults to relative paths if not specified.
                     */
                    schemaModuleSpecifier?: string | undefined;
                    /**
                     * Mapping from GraphQL scalar types to TypeScript types.
                     */
                    scalarTypes?: Record<string, string> | undefined;
                    /**
                     * Whether operation is exported as a default export.
                     * Effective only when a document contains only one operation.
                     * @default true
                     */
                    defaultExportForOperation?: boolean | undefined;
                    /**
                     * Config related to generated names.
                     */
                    name?:
                      | {
                          /**
                           * Suffix for type of operation result.
                           * @default "Result"
                           */
                          operationResultTypeSuffix?: string | undefined;
                          /**
                           * Suffix for type of variables for an operation.
                           */
                          variablesTypeSuffix?: string | undefined;
                          /**
                           * Whether operation name should be capitalized.
                           * @default true
                           */
                          capitalizeOperationNames?: boolean | undefined;
                          /**
                           * Suffix for variable of query.
                           * @default "Query"
                           */
                          queryVariableSuffix?: string | undefined;
                          /**
                           * Suffix for variable of mutation.
                           * @default "Mutation"
                           */
                          mutationVariableSuffix?: string | undefined;
                          /**
                           * Suffix for variable of subscription.
                           * @default "Subscription"
                           */
                          subscriptionVariableSuffix?: string | undefined;
                        }
                      | undefined;
                  }
                | undefined;
            }
          | undefined;
      })
    | undefined;
};
