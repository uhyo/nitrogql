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

Using multiple transformers should be possible by using the [jest-chain-transform](https://www.npmjs.com/package/jest-chain-transform) package:

```js
{
  "transform": {
    // For example, if you are using ts-jest...
    "^.+\.tsx?": "ts-jest",
    // Then, use jest-chain-transform to chain @nitrogql/jest-transform
    // with ts-jest
    "^.+\\.graphql$": ["jest-chain-transform", {
      "transforms": [
        ["@nitrogql/jest-transform", {
          "configFile":  path.resolve(__dirname, "nitrogql.config.js")
        }],
        "ts-jest"
      ]
    }]
  }
}
```
