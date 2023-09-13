import { ApolloServer, HeaderMap, HTTPGraphQLRequest } from "@apollo/server";
import { readFile } from "fs/promises";
import { glob } from "glob";
import {
  getTodoBody,
  getTodos,
  getTodoTags,
  Todo,
  toggleTodo,
} from "./todoMaster";
import { fileURLToPath } from "url";
import { dirname, join } from "path";
import { ResolverOutput, Resolvers } from "@/generated/resolvers";

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

  const todoResolvers: Resolvers<{}>["Todo"] = {
    body(todo) {
      return getTodoBody(todo.id);
    },
    tags(todo) {
      return getTodoTags(todo.id);
    },
  };

  const resolvers: Resolvers<{}> = {
    Query: {
      todos(_parent, variables) {
        if (variables.filter?.unfinishedOnly) {
          return getTodos()
            .filter((todo) => todo.finishedAt === null)
            .map(todoForResolver);
        } else {
          return getTodos().map(todoForResolver);
        }
      },
    },
    Mutation: {
      toggleTodo(_parent, variables) {
        return todoForResolver(toggleTodo(variables.id, variables.finished));
      },
    },
    Todo: todoResolvers,
  };

  const server = new ApolloServer({
    typeDefs: schemas,
    resolvers,
  });
  await server.start();
  return server;
}

function todoForResolver(todo: Todo): ResolverOutput<"Todo"> {
  return {
    id: todo.id,
    createdAt: todo.createdAt.toISOString(),
    finishedAt: todo.finishedAt?.toISOString() ?? null,
  };
}
