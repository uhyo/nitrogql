# nitrogql + Next.js Example

This project shows how to use nitrogql in your Next.js App.

## How to run

```sh
npm install --workspaces=false
npm run generate
npm run dev
```

## Description

This app shows a list of Pokémons fetched from [PokéAPI](https://pokeapi.co/). This example shows how to use nitrogql with an external GraphQL API.

When working with an external GraphQL API, you need to provide a GraphQL schema for nitrogql to generate types from. In this example, an introspection JSON is committed and placed in `src/schema/pokeapi.json`. In a real-world project, you should use a tool that can automatically fetch the schema from the API and generate the introspection JSON (sorry, nitrogql does not provide such a tool yet).

Client-side code benefits from nitrogql's type generation and static check. In this example, there is one GraphQL operation in `src/pokemonList.graphql`. By running `npm run generate`, nitrogql will generate types for this file and put them next to the original file. The generated file is named with a `.d.graphql.ts` suffix.

Behind the scenes, nitrogql's rollup plugin (`@nitrogql/rollup-plugin`) is used for importing GraphQL files in the client-side code. The loader will generate a GraphQL document node from the GraphQL file and export it. The generated document node can be used with GraphQL clients like Apollo Client.

## License Information

Introspection JSON in this example is from [PokéAPI](https://pokeapi.co/).