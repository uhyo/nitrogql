# `@nitrogql/jest-transform`

This package serves as a custom Jest transformer for nitrogql. It is responsible for transforming `.graphql` files into JavaScript modules that export the parsed GraphQL document.

Example Jest configuration:

```js
{
  "transform": {
    "^.+\\.graphql$": ["@nitrogql/jest-transform", {
      "configFile":  path.resolve(__dirname, "nitrogql.config.js")
    }]
  }
}
```

## CommonJS Support

`@nitrogql/jest-transform` is only capable of emitting ES modules. If you are running CommonJS modules during your tests, you will need to use another transformer (for example, `babel-jest` or `ts-jest`) to further transform the output from ES modules to CommonJS.

`@nitrogql/jest-transform` lets you apply another transformer to the output. To do so, use the `additionalTransformer` and `additionalTransformerFilenameSuffix` option:

```js
{
  "transform": {
    // For example, if you are using ts-jest...
    "^.+\.tsx?": ["ts-jest", { isolatedModules: true }],
    "^.+\\.graphql$": ["@nitrogql/jest-transform", {
      "configFile":  path.resolve(__dirname, "nitrogql.config.yml"),
      // Then, use the additionalTransformer option to apply ts-jest to the output.
      "additionalTransformer": ["ts-jest", { isolatedModules: true }],
      // ts-jest expects .ts files, so we need to change the file extension
      // by applying the additionalTransformerFilenameSuffix option.
      "additionalTransformerFilenameSuffix": ".ts"
    }]
  },
}
```
