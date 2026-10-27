import type { Resolvers } from "@/generated/resolvers";
import type { Database } from "../data/db";
import type { IssueEventRecord, NotificationRecord } from "../data/records";
import type { Context } from "./context";

type R = Resolvers<Context>;

type EventOf<T extends IssueEventRecord["__typename"]> = Extract<
  IssueEventRecord,
  { __typename: T }
>;
const event = <T extends IssueEventRecord["__typename"]>(
  db: Database,
  _typename: T,
  id: string,
) => db.get("events", id) as EventOf<T>;

const eventBase = {
  actor: (event: { id: string }, _args: unknown, { db }: Context) =>
    db.get("users", db.get("events", event.id).actorId),
  issue: (event: { id: string }, _args: unknown, { db }: Context) =>
    db.get("issues", db.get("events", event.id).issueId),
};

const userOrNull = (db: Database, id: string | null) =>
  id === null ? null : db.get("users", id);
const projectOrNull = (db: Database, id: string | null) =>
  id === null ? null : db.get("projects", id);

export const IssueCreatedEvent: R["IssueCreatedEvent"] = eventBase;

export const StateChangedEvent: R["StateChangedEvent"] = {
  ...eventBase,
  fromState: (e, _args, { db }) =>
    db.get("workflowStates", event(db, "StateChangedEvent", e.id).fromStateId),
  toState: (e, _args, { db }) =>
    db.get("workflowStates", event(db, "StateChangedEvent", e.id).toStateId),
};

export const AssigneeChangedEvent: R["AssigneeChangedEvent"] = {
  ...eventBase,
  fromAssignee: (e, _args, { db }) =>
    userOrNull(db, event(db, "AssigneeChangedEvent", e.id).fromAssigneeId),
  toAssignee: (e, _args, { db }) =>
    userOrNull(db, event(db, "AssigneeChangedEvent", e.id).toAssigneeId),
};

export const PriorityChangedEvent: R["PriorityChangedEvent"] = eventBase;

export const LabelsChangedEvent: R["LabelsChangedEvent"] = {
  ...eventBase,
  addedLabels: (e, _args, { db }) =>
    event(db, "LabelsChangedEvent", e.id).addedLabelIds.map((id) =>
      db.get("labels", id),
    ),
  removedLabels: (e, _args, { db }) =>
    event(db, "LabelsChangedEvent", e.id).removedLabelIds.map((id) =>
      db.get("labels", id),
    ),
};

export const ProjectChangedEvent: R["ProjectChangedEvent"] = {
  ...eventBase,
  fromProject: (e, _args, { db }) =>
    projectOrNull(db, event(db, "ProjectChangedEvent", e.id).fromProjectId),
  toProject: (e, _args, { db }) =>
    projectOrNull(db, event(db, "ProjectChangedEvent", e.id).toProjectId),
};

export const CommentedEvent: R["CommentedEvent"] = {
  ...eventBase,
  comment: (e, _args, { db }) =>
    db.get("comments", event(db, "CommentedEvent", e.id).commentId),
};

type NotificationOf<T extends NotificationRecord["__typename"]> = Extract<
  NotificationRecord,
  { __typename: T }
>;
const notification = <T extends NotificationRecord["__typename"]>(
  db: Database,
  _typename: T,
  id: string,
) => db.get("notifications", id) as NotificationOf<T>;

const actor = (n: { id: string }, _args: unknown, { db }: Context) =>
  db.get("users", db.get("notifications", n.id).actorId);

export const IssueAssignedNotification: R["IssueAssignedNotification"] = {
  actor,
  issue: (n, _args, { db }) =>
    db.get(
      "issues",
      notification(db, "IssueAssignedNotification", n.id).issueId,
    ),
};

export const IssueStateChangedNotification: R["IssueStateChangedNotification"] =
  {
    actor,
    issue: (n, _args, { db }) =>
      db.get(
        "issues",
        notification(db, "IssueStateChangedNotification", n.id).issueId,
      ),
    state: (n, _args, { db }) =>
      db.get(
        "workflowStates",
        notification(db, "IssueStateChangedNotification", n.id).stateId,
      ),
  };

export const CommentMentionNotification: R["CommentMentionNotification"] = {
  actor,
  comment: (n, _args, { db }) =>
    db.get(
      "comments",
      notification(db, "CommentMentionNotification", n.id).commentId,
    ),
};

export const CommentReplyNotification: R["CommentReplyNotification"] = {
  actor,
  comment: (n, _args, { db }) =>
    db.get(
      "comments",
      notification(db, "CommentReplyNotification", n.id).commentId,
    ),
};

export const ProjectUpdateNotification: R["ProjectUpdateNotification"] = {
  actor,
  projectUpdate: (n, _args, { db }) =>
    db.get(
      "projectUpdates",
      notification(db, "ProjectUpdateNotification", n.id).projectUpdateId,
    ),
};
