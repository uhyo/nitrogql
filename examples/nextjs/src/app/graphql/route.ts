import { ApolloServer, HeaderMap, HTTPGraphQLRequest } from "@apollo/server";
import {
  getTodoBody,
  getTodos,
  getTodoTags,
  Todo,
  toggleTodo,
} from "./todoMaster";
import { ResolverOutput, Resolvers } from "@/generated/resolvers";
import schema from "../../schema/scalar";
import { schema as typeDefs } from "@/generated/graphql";
import { mergeSchemas } from "@graphql-tools/schema";

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
  const server = new ApolloServer({
    schema: mergeSchemas({
      schemas: [schema],
      typeDefs,
      resolvers,
    }),
  });
  await server.start();
  return server;
}

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

function todoForResolver(todo: Todo): ResolverOutput<"Todo"> {
  return {
    id: todo.id,
    createdAt: todo.createdAt.toISOString(),
    finishedAt: todo.finishedAt?.toISOString() ?? null,
  };
}
