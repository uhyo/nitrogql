"use client";

import { useMutation, useQuery } from "urql";
import Query from "./myTodoList.graphql";
import ToggleTodo from "./toggleTodo.graphql";
import classes from "./TodoList.module.css";
import { Filter } from "../useControls";
import { TodoItem } from "./TodoItem";

export const TodoList: React.FC<{
  filter: Filter;
}> = ({ filter }) => {
  const [{ data }] = useQuery({
    query: Query,
    variables: {
      unfinishedOnly: filter === "unfinished" ? true : null,
    },
  });
  const [, runToggleTodo] = useMutation(ToggleTodo);
  if (data === undefined) {
    return null;
  }
  return (
    <div className={classes.todoList}>
      {data.todos.map((todo) => (
        <TodoItem
          key={todo.id}
          todo={todo}
          onToggle={(finished) => {
            runToggleTodo({
              id: todo.id,
              finished,
            });
          }}
        />
      ))}
    </div>
  );
};
