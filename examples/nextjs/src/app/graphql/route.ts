import { ApolloServer, HeaderMap, HTTPGraphQLRequest } from "@apollo/server";
import { readFile } from "fs/promises";
import { glob } from "glob";
import { getTodos } from "./todoMaster";
import { fileURLToPath } from "url";
import { dirname, join } from "path";

export async function POST(request: Request) {
  server ??= await initServer();
  const url = new URL(request.url, "https://localhost");
  const req: HTTPGraphQLRequest = {
    method: request.method,
    headers: new HeaderMap(request.headers),
    search: url.search,
    body: await request.json(),
  };
  const response = await server.executeHTTPGraphQLRequest({
    httpGraphQLRequest: req,
    async context() {
      return {};
    },
  });
  if (response.body.kind === "complete") {
    return new Response(response.body.string, {
      status: response.status,
      headers: Array.from(response.headers.entries()),
    });
  } else {
    const iterator = response.body.asyncIterator;
    const stream = new ReadableStream<string>({
      async pull(controller) {
        const { value, done } = await iterator.next();

        if (done) {
          controller.close();
        } else {
          controller.enqueue(value);
        }
      },
    });
    return new Response(stream, {
      status: response.status,
      headers: Array.from(response.headers.entries()),
    });
  }
}

let server: ApolloServer | undefined;

async function initServer(): Promise<ApolloServer> {
  const currentDirectory = dirname(fileURLToPath(import.meta.url));
  const schemas = await glob("../../schema/*.graphql", {
    cwd: currentDirectory,
  }).then((files) =>
    Promise.all(
      files.map((file) => readFile(join(currentDirectory, file), "utf-8"))
    )
  );

  const resolvers = {
    Query: {
      todos(_) {
        return getTodos();
      },
    },
  };

  const server = new ApolloServer({
    typeDefs: schemas,
    resolvers,
  });
  await server.start();
  return server;
}
