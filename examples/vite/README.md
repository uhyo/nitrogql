# nitrogql + Vite Example

This project shows how to use nitrogql in your Vite app.

## How to run

```sh
npm install --workspaces=false
npm run generate
npm run dev
```

## Description

This is an example TODO App project that contains both client and server code in one Next.js application.

This is made in a **schema-first** way, meaning that the GraphQL schema (`src/schema/*.graphql`) is written first, and server-side code and client-side code both obey the schema.

Since nitrogql does not provide support for writing server-side code yet, the server-side code is written in a not-very-type-safe way. The server-side code is located in `src/app/graphql`.

Client-side code benefits from nitrogql's type generation and static check. The client-side GraphQL code is located in `src/app/TodoList`. By running `npm run generate`, nitrogql will generate types for these files and put them next to the original files. The generated files are named with a `.d.graphql.ts` suffix.

Behind the scenes, nitrogql's webpack loader (`@nitrogql/graphql-loader`) is used for importing GraphQL files in the client-side code. The loader will generate a GraphQL document node from the GraphQL file and export it. The generated document node can be used with GraphQL clients like Apollo Client.
