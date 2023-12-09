import { TodoItemFragment } from "./TodoItemFragment.graphql";
import classes from "../TodoList.module.css";

export const TodoItem: React.FC<{
  todo: TodoItemFragment;
  onToggle: (finished: boolean) => void;
}> = ({ todo, onToggle }) => {
  return (
    <div className={classes.item}>
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
          onClick={() => onToggle(true)}
        >
          ⬜️
        </button>
      ) : (
        <button
          type="button"
          aria-label="Mark as undone"
          onClick={() => onToggle(false)}
        >
          ✅
        </button>
      )}
    </div>
  );
};
