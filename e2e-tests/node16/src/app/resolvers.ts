import type { ResolverOutput, Resolvers } from "../generated/resolvers.js";

type Context = {};

const resolvers: Resolvers<Context> = {
  Query: {
    me: (): ResolverOutput<"User"> => {
      return {
        id: "1",
        name: "John Doe",
        email: "john@example.com",
      };
    },
    news: () => {
      return [
        {
          id: "1",
          title: "News 1",
          content: "Content 1",
          publishedAt: new Date(),
        },
        {
          id: "2",
          title: "News 2",
          content: "Content 2",
          publishedAt: new Date(),
        },
      ];
    },
  },
};

console.log(resolvers);
