import { ApolloServer, HeaderMap, HTTPGraphQLRequest } from "@apollo/server";
import { makeExecutableSchema } from "@graphql-tools/schema";
import { schema as typeDefs } from "@/generated/graphql";
import { getDatabase } from "@/server/data/db";
import { type Context, resolvers } from "@/server/resolvers";

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
      const db = getDatabase();
      // There is no authentication in this example; the first user is always signed in.
      return { db, viewerId: db.data.viewerId };
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

let server: ApolloServer<Context> | undefined;

async function initServer(): Promise<ApolloServer<Context>> {
  const server = new ApolloServer<Context>({
    schema: makeExecutableSchema({
      typeDefs,
      resolvers,
    }),
  });
  await server.start();
  return server;
}
