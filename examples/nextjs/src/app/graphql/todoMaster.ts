export type Todo = {
  id: string;
  tags: readonly string[];
  createdAt: Date;
  finishedAt: Date | null;
};

export type Tag = {
  readonly label: string;
  readonly color: string;
};

const tagMaster = new Map<string, Tag>([
  ["housework", { label: "housework 🏠", color: "#fac5f4" }],
  ["business", { label: "business 🏢", color: "#c5ccfa" }],
  ["healthcare", { label: "healthcare 🍻", color: "#c6fac5" }],
  ["eat", { label: "eat 🍽", color: "#faf1c5" }],
]);

const todoMaster: Todo[] = [
  {
    id: "1",
    tags: ["eat"],
    createdAt: new Date("2023-03-21T09:00"),
    finishedAt: null,
  },
  {
    id: "2",
    tags: ["eat", "housework"],
    createdAt: new Date("2023-03-21T12:00"),
    finishedAt: null,
  },
  {
    id: "3",
    tags: ["healthcare", "business"],
    createdAt: new Date("2023-03-21T13:00"),
    finishedAt: null,
  },
  {
    id: "4",
    tags: ["business"],
    createdAt: new Date("2023-03-21T15:00"),
    finishedAt: null,
  },
  {
    id: "5",
    tags: ["housework"],
    createdAt: new Date("2023-03-21T17:00"),
    finishedAt: null,
  },
  {
    id: "6",
    tags: ["eat", "healthcare", "business"],
    createdAt: new Date("2023-03-21T18:00"),
    finishedAt: null,
  },
];
const todoBodyData = new Map<string, string>(
  todoMaster.map((todo, index) => [
    todo.id,
    [
      "Eat breakfast",
      "Eat Lunch",
      "Eat snacks",
      "Eat snacks",
      "Go shopping for dinner",
      "Eat dinner",
    ][index],
  ])
);

/**
 * @returns Get current Todos.
 */
export const getTodos = () => todoMaster.concat([]);

/**
 * Get body of a Todo.
 */
export const getTodoBody = (id: string) => {
  const body = todoBodyData.get(id);
  if (body === undefined) {
    throw new Error(`Cannot find TODO of ID '${id}'`);
  }
  return body;
};

/**
 * Get tags of a Todo.
 */
export const getTodoTags = (id: string) => {
  const todo = todoMaster.find((todo) => todo.id === id);
  if (todo === undefined) {
    throw new Error(`Cannot find TODO of ID '${id}'`);
  }
  return todo.tags.map((id) => {
    const tag = tagMaster.get(id);
    if (tag === undefined) {
      throw new Error(`Cannot find Tag of ID '${id}'`);
    }
    return {
      id,
      label: tag.label,
      color: tag.color,
    };
  });
};

/**
 * Toggle state of a Todo.
 */
export const toggleTodo = (id: string, finished: boolean): Todo => {
  const todo = todoMaster.find((todo) => todo.id === id);
  if (todo === undefined) {
    throw new Error(`Cannot find TODO of ID '${id}'`);
  }

  if (finished) {
    todo.finishedAt = new Date();
  } else {
    todo.finishedAt = null;
  }

  return todo;
};
