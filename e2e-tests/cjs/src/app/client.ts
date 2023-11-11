import { ApolloClient, InMemoryCache } from "@apollo/client";
import TopPageQuery from "./operation.graphql";

const apolloClient = new ApolloClient({
  cache: new InMemoryCache(),
});

async function main() {
  const result = await apolloClient.query({
    query: TopPageQuery,
  });
  const { me, news } = result.data;
  if (me !== null) {
    const myId: string = me.id;
    const myName: string = me.name;
    console.log({ myId, myName });
  }
  for (const n of news) {
    const id: string = n.id;
    const title: string = n.title;
    const content: string = n.content;
    const publishedAt: Date | string = n.publishedAt;
    console.log({ id, title, content, publishedAt });
  }
}
