import type { ResolverOutput, Resolvers } from "@/generated/resolvers";
import type { IssueRecord } from "../data/records";
import type { Context } from "./context";

export const Mutation: Resolvers<Context>["Mutation"] = {
  createIssue(_parent, { input }, { db, viewerId }) {
    const team = db.get("teams", input.teamId);
    const states = db.data.workflowStates.filter(
      (state) => state.teamId === team.id,
    );
    const now = new Date().toISOString();
    const issue = db.insert("issues", {
      teamId: team.id,
      number:
        Math.max(
          0,
          ...db.data.issues
            .filter((issue) => issue.teamId === team.id)
            .map((issue) => issue.number),
        ) + 1,
      title: input.title,
      description: input.description ?? null,
      priority: input.priority ?? "NO_PRIORITY",
      estimate: input.estimate ?? null,
      dueDate: input.dueDate ?? null,
      createdAt: now,
      updatedAt: now,
      completedAt: null,
      canceledAt: null,
      archivedAt: null,
      stateId:
        input.stateId ??
        states.find((state) => state.type === "TRIAGE")?.id ??
        states[0].id,
      assigneeId: input.assigneeId ?? null,
      creatorId: viewerId,
      labelIds: [...(input.labelIds ?? [])],
      projectId: input.projectId ?? null,
      milestoneId: null,
      cycleId: input.cycleId ?? null,
      parentId: input.parentId ?? null,
      subscriberIds: [viewerId],
    });
    db.recordEvent(issue, {
      __typename: "IssueCreatedEvent",
      actorId: viewerId,
    });
    return issue;
  },
  updateIssue(_parent, { id, input }, { db, viewerId }) {
    const issue = db.get("issues", id);
    if (input.stateId) {
      db.setIssueState(issue, input.stateId, viewerId);
    }
    if (
      input.assigneeId !== undefined &&
      input.assigneeId !== issue.assigneeId
    ) {
      db.recordEvent(issue, {
        __typename: "AssigneeChangedEvent",
        actorId: viewerId,
        fromAssigneeId: issue.assigneeId,
        toAssigneeId: input.assigneeId,
      });
      issue.assigneeId = input.assigneeId;
    }
    if (input.priority && input.priority !== issue.priority) {
      db.recordEvent(issue, {
        __typename: "PriorityChangedEvent",
        actorId: viewerId,
        fromPriority: issue.priority,
        toPriority: input.priority,
      });
      issue.priority = input.priority;
    }
    if (input.projectId !== undefined && input.projectId !== issue.projectId) {
      db.recordEvent(issue, {
        __typename: "ProjectChangedEvent",
        actorId: viewerId,
        fromProjectId: issue.projectId,
        toProjectId: input.projectId,
      });
      issue.projectId = input.projectId;
      issue.milestoneId = null;
    }
    const plainFields = [
      "title",
      "description",
      "milestoneId",
      "cycleId",
      "estimate",
      "dueDate",
    ] as const satisfies (keyof IssueRecord & keyof typeof input)[];
    for (const field of plainFields) {
      const value = input[field];
      if (value !== undefined && (field !== "title" || value !== null)) {
        (issue as Record<typeof field, unknown>)[field] = value;
        issue.updatedAt = new Date().toISOString();
      }
    }
    return issue;
  },
  addIssueLabel(_parent, { issueId, labelId }, { db, viewerId }) {
    const issue = db.get("issues", issueId);
    if (!issue.labelIds.includes(labelId)) {
      db.get("labels", labelId);
      issue.labelIds.push(labelId);
      db.recordEvent(issue, {
        __typename: "LabelsChangedEvent",
        actorId: viewerId,
        addedLabelIds: [labelId],
        removedLabelIds: [],
      });
    }
    return issue;
  },
  removeIssueLabel(_parent, { issueId, labelId }, { db, viewerId }) {
    const issue = db.get("issues", issueId);
    if (issue.labelIds.includes(labelId)) {
      issue.labelIds = issue.labelIds.filter((id) => id !== labelId);
      db.recordEvent(issue, {
        __typename: "LabelsChangedEvent",
        actorId: viewerId,
        addedLabelIds: [],
        removedLabelIds: [labelId],
      });
    }
    return issue;
  },
  archiveIssue(_parent, { id }, { db }) {
    const issue = db.get("issues", id);
    issue.archivedAt ??= new Date().toISOString();
    return issue;
  },
  setIssueSubscription(_parent, { issueId, subscribed }, { db, viewerId }) {
    const issue = db.get("issues", issueId);
    issue.subscriberIds = issue.subscriberIds.filter((id) => id !== viewerId);
    if (subscribed) {
      issue.subscriberIds.push(viewerId);
    }
    return issue;
  },
  addIssueRelation(_parent, { issueId, relatedIssueId, type }, { db }) {
    const issue = db.get("issues", issueId);
    db.get("issues", relatedIssueId);
    db.insert("relations", { issueId, relatedIssueId, type });
    const inverse = { BLOCKS: "BLOCKED_BY", BLOCKED_BY: "BLOCKS" } as const;
    if (type === "BLOCKS" || type === "BLOCKED_BY") {
      db.insert("relations", {
        issueId: relatedIssueId,
        relatedIssueId: issueId,
        type: inverse[type],
      });
    }
    return issue;
  },
  createComment(_parent, { input }, { db, viewerId }) {
    const issue = db.get("issues", input.issueId);
    const comment = db.insert("comments", {
      issueId: issue.id,
      authorId: viewerId,
      parentId: input.parentId ?? null,
      body: input.body,
      createdAt: new Date().toISOString(),
      editedAt: null,
    });
    db.recordEvent(issue, {
      __typename: "CommentedEvent",
      actorId: viewerId,
      commentId: comment.id,
    });
    if (!issue.subscriberIds.includes(viewerId)) {
      issue.subscriberIds.push(viewerId);
    }
    return comment;
  },
  updateComment(_parent, { id, body }, { db, viewerId }) {
    const comment = db.get("comments", id);
    if (comment.authorId !== viewerId) {
      throw new Error("You cannot edit this comment");
    }
    comment.body = body;
    comment.editedAt = new Date().toISOString();
    return comment;
  },
  deleteComment(_parent, { id }, { db, viewerId }) {
    const comment = db.get("comments", id);
    if (comment.authorId !== viewerId) {
      throw new Error("You cannot delete this comment");
    }
    for (const reply of db.data.comments.filter((c) => c.parentId === id)) {
      reply.parentId = null;
    }
    db.remove("comments", id);
    for (const event of db.data.events.filter(
      (event) =>
        event.__typename === "CommentedEvent" && event.commentId === id,
    )) {
      db.remove("events", event.id);
    }
    return db.get("issues", comment.issueId);
  },
  toggleReaction(_parent, { subjectId, emoji }, { db, viewerId }) {
    const subject = db.findAny(subjectId);
    if (
      subject === undefined ||
      !["issues", "comments", "projectUpdates"].includes(subject.table)
    ) {
      throw new Error(`Cannot react to '${subjectId}'`);
    }
    const reactions = db.reactionsOf(subjectId);
    const userIds = reactions.get(emoji) ?? [];
    reactions.set(
      emoji,
      userIds.includes(viewerId)
        ? userIds.filter((id) => id !== viewerId)
        : [...userIds, viewerId],
    );
    return subject.record as ResolverOutput<"ReactionTarget">;
  },
  createProjectUpdate(_parent, { input }, { db, viewerId }) {
    const project = db.get("projects", input.projectId);
    return db.insert("projectUpdates", {
      projectId: project.id,
      authorId: viewerId,
      body: input.body,
      health: input.health,
      createdAt: new Date().toISOString(),
    });
  },
  updateProject(_parent, { id, input }, { db }) {
    const project = db.get("projects", id);
    if (input.name) {
      project.name = input.name;
    }
    if (input.description !== undefined) {
      project.description = input.description;
    }
    if (input.status) {
      project.status = input.status;
    }
    if (input.leadId !== undefined) {
      project.leadId = input.leadId;
    }
    if (input.targetDate !== undefined) {
      project.targetDate = input.targetDate;
    }
    return project;
  },
  markNotificationRead(_parent, { id, read }, { db, viewerId }) {
    const notification = db.get("notifications", id);
    if (notification.userId !== viewerId) {
      throw new Error(`Cannot find notification of ID '${id}'`);
    }
    notification.readAt = read
      ? (notification.readAt ?? new Date().toISOString())
      : null;
    return notification;
  },
  markAllNotificationsRead(_parent, _args, { db, viewerId }) {
    const now = new Date().toISOString();
    let count = 0;
    for (const notification of db.data.notifications) {
      if (notification.userId === viewerId && notification.readAt === null) {
        notification.readAt = now;
        count++;
      }
    }
    return count;
  },
  updateViewerStatus(_parent, { input }, { db, viewerId }) {
    const viewer = db.get("users", viewerId);
    viewer.status = input
      ? {
          emoji: input.emoji,
          message: input.message,
          expiresAt: input.expiresAt ?? null,
        }
      : null;
    return viewer;
  },
};
