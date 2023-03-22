"use client";

import { useMutation, useQuery } from "urql";
import Query from "./myTodoList.graphql";
import ToggleTodo from "./toggleTodo.graphql";
import classes from "./TodoList.module.css";
import { Filter } from "../useControls";

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
        <div key={todo.id} className={classes.item}>
          <p>{todo.body}</p>
          <div>
            {todo.tags.map((tag) => (
              <span key={tag.id} style={{ background: tag.color }}>
                {tag.label}
              </span>
            ))}
          </div>
          <div>
            {todo.finishedAt !== null ? (
              <>
                Finished at{" "}
                <time dateTime={new Date(todo.finishedAt).toISOString()}>
                  {new Date(todo.finishedAt).toLocaleString()}
                </time>
              </>
            ) : (
              <>
                Created at{" "}
                <time dateTime={new Date(todo.createdAt).toISOString()}>
                  {new Date(todo.createdAt).toLocaleString()}
                </time>
              </>
            )}
          </div>
          {todo.finishedAt === null ? (
            <button
              type="button"
              aria-label="Mark as done"
              onClick={() =>
                runToggleTodo({
                  id: todo.id,
                  finished: true,
                })
              }
            >
              ⬜️
            </button>
          ) : (
            <button
              type="button"
              aria-label="Mark as undone"
              onClick={() =>
                runToggleTodo({
                  id: todo.id,
                  finished: false,
                }).then(() => {
                  debugger;
                })
              }
            >
              ✅
            </button>
          )}
        </div>
      ))}
    </div>
  );
};
