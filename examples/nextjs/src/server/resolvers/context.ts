import type { Database, TableName } from "../data/db";

export type Context = {
  db: Database;
  /** ID of the signed-in user. */
  viewerId: string;
};

const typenames = {
  users: "User",
  teams: "Team",
  workflowStates: "WorkflowState",
  labels: "Label",
  projects: "Project",
  milestones: "Milestone",
  projectUpdates: "ProjectUpdate",
  cycles: "Cycle",
  issues: "Issue",
  comments: "Comment",
  relations: "IssueRelation",
  attachments: "Attachment",
} as const satisfies Partial<Record<TableName, string>>;

/**
 * Creates a `__resolveType` resolver for an abstract type whose possible types are `T`.
 * Types are determined by IDs; events and notifications have `__typename` in records.
 */
export function resolveTypeById<T extends string>() {
  return (object: { id: string }, { db }: Context): T => {
    const found = db.findAny(object.id);
    if (found === undefined) {
      throw new Error(`Cannot find object of ID '${object.id}'`);
    }
    if (found.table === "events" || found.table === "notifications") {
      return (found.record as { __typename: T }).__typename;
    }
    return typenames[found.table] as T;
  };
}
