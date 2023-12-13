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
        nitrogql?: NitrogqlExtension | undefined;
      })
    | undefined;
};

/**
 * Nitrogql's config object.
 */
export type NitrogqlExtension = {
  /**
   * List of plugins to use.
   */
  plugins?: readonly string[] | undefined;
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
         * Needed if you want to generate schema types.
         */
        schemaOutput?: string | undefined;
        /**
         * Path to the output GraphQl source file for use by a GraphQL server.
         * Allows you to emit processed GraphQL source as one string.
         * Note: output is a `.ts` file so you can use it as a module.
         */
        serverGraphqlOutput?: string | undefined;
        /**
         * Path to the output resolvers type definition file.
         * Needed if you want to generate resolvers types.
         */
        resolversOutput?: string | undefined;
        /**
         * Module specifier for importing schema types from operations.
         * Defaults to relative paths if not specified.
         */
        schemaModuleSpecifier?: string | undefined;
        /**
         * Config related to generated types.
         */
        type?:
          | {
              /**
               * Mapping from GraphQL scalar types to TypeScript types.
               */
              scalarTypes?: Record<string, string> | undefined;
              /**
               * Whether to allow undefined as input value
               * for nullable fields.
               * @default true
               */
              allowUndefinedAsOptionalInput?: boolean | undefined;
            }
          | undefined;
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
               * Suffix for type of fragment.
               */
              fragmentTypeSuffix?: string | undefined;
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
        /**
         * Config related to exporting generated names.
         */
        export?:
          | {
              /**
               * Whether operation is exported as a default export.
               * Effective only when a document contains only one operation.
               * @default true
               */
              defaultExportForOperation?: boolean | undefined;
              /**
               * Whether operation result type is exported.
               * @default false
               */
              operationResultType?: boolean | undefined;
              /**
               * Whether operation variables type is exported.
               * @default false
               */
              variablesType?: boolean | undefined;
            }
          | undefined;
        /**
         * Whether to generate runtime code for schema types.
         * If true, an object is emitted for each enum.
         * @default false
         */
        emitSchemaRuntime?: boolean | undefined;
      }
    | undefined;
};
