"use client";

import { useQuery } from "urql";
import Query from "./myTodoList.graphql";

export const TodoList: React.FC = () => {
  const [data] = useQuery({ query: Query });
  return <p>TODO: make todos</p>;
};
