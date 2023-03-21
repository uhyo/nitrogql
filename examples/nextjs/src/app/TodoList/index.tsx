"use client";

import { useQuery } from "urql";
import Query from "./myTodoList.graphql";
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
            <div>
              Created at{" "}
              <time dateTime={new Date(todo.createdAt).toISOString()}>
                {new Date(todo.createdAt).toLocaleString()}
              </time>
            </div>
            {todo.finishedAt !== null && (
              <div>
                Finished at{" "}
                <time dateTime={new Date(todo.finishedAt).toISOString()}>
                  {new Date(todo.finishedAt).toLocaleString()}
                </time>
              </div>
            )}
          </div>
          <button type="button" aria-label="Mark as done">
            ⬜️
          </button>
        </div>
      ))}
    </div>
  );
};
