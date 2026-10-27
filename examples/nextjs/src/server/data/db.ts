import type {
  CommentRecord,
  IssueEventRecord,
  IssuePriority,
  IssueRecord,
  ReactionsRecord,
  WorkflowStateType,
} from "./records";
import { seed, type SeedData } from "./seed";

/**
 * In-memory database. Data is reset when the server restarts.
 */
export class Database {
  readonly data: SeedData;
  #byId = new Map<string, unknown>();
  #nextId = 1;

  constructor(now: Date) {
    this.data = seed(now);
    const { reactions: _, viewerId: __, ...tables } = this.data;
    for (const table of Object.values(tables)) {
      for (const record of table) {
        this.#byId.set(record.id, record);
      }
    }
  }

  /** Inserts a record into a table. */
  insert<K extends TableName>(
    table: K,
    record: Omit<SeedData[K][number], "id">,
  ): SeedData[K][number] {
    const id = `${idPrefixes[table]}_new${this.#nextId++}`;
    const inserted = { ...record, id } as SeedData[K][number];
    (this.data[table] as { id: string }[]).push(inserted);
    this.#byId.set(id, inserted);
    return inserted;
  }

  /** Removes a record from a table. */
  remove(table: TableName, id: string) {
    const records = this.data[table] as { id: string }[];
    const index = records.findIndex((record) => record.id === id);
    if (index >= 0) {
      records.splice(index, 1);
      this.#byId.delete(id);
    }
  }

  /** Finds a record by ID. Throws if not found. */
  get<K extends TableName>(table: K, id: string): SeedData[K][number] {
    const record = this.find(table, id);
    if (record === undefined) {
      throw new Error(`Cannot find ${table} of ID '${id}'`);
    }
    return record;
  }

  /** Finds a record by ID. */
  find<K extends TableName>(
    table: K,
    id: string,
  ): SeedData[K][number] | undefined {
    if (!id.startsWith(`${idPrefixes[table]}_`)) {
      return undefined;
    }
    return this.#byId.get(id) as SeedData[K][number] | undefined;
  }

  /** Finds a record of any table by ID. */
  findAny(id: string): { table: TableName; record: unknown } | undefined {
    const record = this.#byId.get(id);
    if (record === undefined) {
      return undefined;
    }
    const prefix = id.slice(0, id.indexOf("_"));
    const table = (Object.keys(idPrefixes) as TableName[]).find(
      (table) => idPrefixes[table] === prefix,
    )!;
    return { table, record };
  }

  reactionsOf(subjectId: string): ReactionsRecord {
    let reactions = this.data.reactions.get(subjectId);
    if (reactions === undefined) {
      reactions = new Map();
      this.data.reactions.set(subjectId, reactions);
    }
    return reactions;
  }

  stateTypeOf(issue: IssueRecord): WorkflowStateType {
    return this.get("workflowStates", issue.stateId).type;
  }

  identifierOf(issue: IssueRecord): string {
    return `${this.get("teams", issue.teamId).key}-${issue.number}`;
  }

  findIssueByIdentifier(identifier: string): IssueRecord | undefined {
    const match = /^([A-Za-z]+)-(\d+)$/.exec(identifier);
    if (match === null) {
      return undefined;
    }
    const team = this.data.teams.find(
      (team) => team.key === match[1].toUpperCase(),
    );
    return this.data.issues.find(
      (issue) => issue.teamId === team?.id && issue.number === Number(match[2]),
    );
  }

  /** Comments and events of an issue, oldest first. */
  timelineOf(issueId: string): (CommentRecord | IssueEventRecord)[] {
    const events = this.data.events.filter(
      (event) =>
        event.issueId === issueId && event.__typename !== "CommentedEvent",
    );
    const comments = this.data.comments.filter(
      (comment) => comment.issueId === issueId && comment.parentId === null,
    );
    return [...events, ...comments].sort((a, b) =>
      a.createdAt.localeCompare(b.createdAt),
    );
  }

  /** Records an event and updates `updatedAt` of the issue. */
  recordEvent(
    issue: IssueRecord,
    event: DistributiveOmit<IssueEventRecord, "id" | "issueId" | "createdAt">,
  ) {
    const now = new Date().toISOString();
    this.insert("events", {
      ...event,
      issueId: issue.id,
      createdAt: now,
    } as Omit<IssueEventRecord, "id">);
    issue.updatedAt = now;
  }

  /** Sets the state of an issue, updating completion dates. */
  setIssueState(issue: IssueRecord, stateId: string, actorId: string) {
    if (issue.stateId === stateId) {
      return;
    }
    const state = this.get("workflowStates", stateId);
    if (state.teamId !== issue.teamId) {
      throw new Error("The state belongs to another team");
    }
    this.recordEvent(issue, {
      __typename: "StateChangedEvent",
      actorId,
      fromStateId: issue.stateId,
      toStateId: stateId,
    });
    issue.stateId = stateId;
    issue.completedAt = state.type === "COMPLETED" ? issue.updatedAt : null;
    issue.canceledAt = state.type === "CANCELED" ? issue.updatedAt : null;
  }
}

export type TableName = Exclude<keyof SeedData, "viewerId" | "reactions">;

const idPrefixes: Record<TableName, string> = {
  users: "usr",
  teams: "team",
  workflowStates: "ws",
  labels: "lbl",
  projects: "prj",
  milestones: "ms",
  projectUpdates: "pu",
  cycles: "cyc",
  issues: "iss",
  events: "evt",
  comments: "cmt",
  relations: "rel",
  attachments: "att",
  notifications: "ntf",
};

export const priorityOrder: Record<IssuePriority, number> = {
  URGENT: 0,
  HIGH: 1,
  MEDIUM: 2,
  LOW: 3,
  NO_PRIORITY: 4,
};

type DistributiveOmit<T, K extends PropertyKey> = T extends unknown
  ? Omit<T, K>
  : never;

let db: Database | undefined;

/** Returns the database, creating it on first use. */
export function getDatabase(): Database {
  db ??= new Database(new Date());
  return db;
}
