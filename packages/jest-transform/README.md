# `@nitrogql/jest-transform`

This package serves as a custom Jest transformer for NitroGQL. It is responsible for transforming `.graphql` files into JavaScript modules that export the parsed GraphQL document.

Example Jest configuration:

```json
{
  "transform": {
    "^.+\\.graphql$": "@nitrogql/jest-transform"
  }
}
```
