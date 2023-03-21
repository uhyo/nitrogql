import { randomUUID } from "crypto";

type Todo = {
  id: string;
  body: string;
  tags: readonly Tag[];
  createdAt: Date;
  finishedAt: Date | null;
};

type Tag = {
  readonly id: string;
  readonly label: string;
  readonly color: string;
};

const tags = {
  housework: {
    id: randomUUID(),
    label: "housework 🏠",
    color: "#fac5f4",
  },
  business: {
    id: randomUUID(),
    label: "business 🏢",
    color: "#c5ccfa",
  },
  healthcare: {
    id: randomUUID(),
    label: "healthcare 🍻",
    color: "#c6fac5",
  },
  eat: {
    id: randomUUID(),
    label: "eat 🍽",
    color: "#faf1c5",
  },
} satisfies Record<string, Tag>;

const todoMaster: Todo[] = [
  {
    id: randomUUID(),
    body: "Eat breakfast",
    tags: [tags.eat],
    createdAt: new Date("2023-03-21T09:00"),
    finishedAt: null,
  },
  {
    id: randomUUID(),
    body: "Eat Lunch",
    tags: [tags.eat, tags.housework],
    createdAt: new Date("2023-03-21T12:00"),
    finishedAt: null,
  },
  {
    id: randomUUID(),
    body: "Eat snacks",
    tags: [tags.healthcare, tags.business],
    createdAt: new Date("2023-03-21T13:00"),
    finishedAt: null,
  },
  {
    id: randomUUID(),
    body: "Eat snacks",
    tags: [tags.business],
    createdAt: new Date("2023-03-21T15:00"),
    finishedAt: null,
  },
  {
    id: randomUUID(),
    body: "Go shopping for dinner",
    tags: [tags.housework],
    createdAt: new Date("2023-03-21T17:00"),
    finishedAt: null,
  },
  {
    id: randomUUID(),
    body: "Eat dinner",
    tags: [tags.eat, tags.healthcare, tags.business],
    createdAt: new Date("2023-03-21T18:00"),
    finishedAt: null,
  },
];

/**
 * @returns Get current Todos.
 */
export const getTodos = () => todoMaster.concat([]);

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
