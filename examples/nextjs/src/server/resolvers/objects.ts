import type { Resolvers } from "@/generated/resolvers";
import type { Database } from "../data/db";
import type { ReactionGroupModel } from "../data/records";
import type { Context } from "./context";
import {
  issueConnection,
  paginate,
  progressOf,
  reactionGroups,
} from "./helpers";

type R = Resolvers<Context>;

const reactionUsers = (db: Database, group: ReactionGroupModel) =>
  db.reactionsOf(group.subjectId).get(group.emoji) ?? [];

export const User: R["User"] = {
  status(user, _args, { db }) {
    const { status } = db.get("users", user.id);
    if (status?.expiresAt && status.expiresAt < new Date().toISOString()) {
      return null;
    }
    return status;
  },
  teams(user, _args, { db }) {
    return db.data.teams.filter((team) =>
      team.members.some((member) => member.userId === user.id),
    );
  },
  assignedIssues(user, args, { db }) {
    return issueConnection(
      db,
      db.data.issues.filter((issue) => issue.assigneeId === user.id),
      args,
    );
  },
  createdIssues(user, args, { db }) {
    return issueConnection(
      db,
      db.data.issues.filter((issue) => issue.creatorId === user.id),
      args,
    );
  },
  subscribedIssues(user, args, { db }) {
    return issueConnection(
      db,
      db.data.issues.filter((issue) => issue.subscriberIds.includes(user.id)),
      args,
    );
  },
  recentActivity(user, { first }, { db }) {
    return db.data.events
      .filter((event) => event.actorId === user.id)
      .sort((a, b) => b.createdAt.localeCompare(a.createdAt))
      .slice(0, first ?? 20);
  },
};

export const Team: R["Team"] = {
  members(team, _args, { db }) {
    return db.get("teams", team.id).members;
  },
  workflowStates(team, _args, { db }) {
    return db.data.workflowStates
      .filter((state) => state.teamId === team.id)
      .sort((a, b) => a.position - b.position);
  },
  labels(team, _args, { db }) {
    return db.data.labels.filter((label) => label.teamId === team.id);
  },
  projects(team, { status }, { db }) {
    return db.data.projects.filter(
      (project) =>
        project.teamId === team.id &&
        (status == null || status.includes(project.status)),
    );
  },
  cycles(team, { includeCompleted }, { db }) {
    return db.data.cycles
      .filter(
        (cycle) =>
          cycle.teamId === team.id &&
          (includeCompleted || cycle.completedAt === null),
      )
      .sort((a, b) => b.number - a.number);
  },
  activeCycle(team, _args, { db }) {
    const now = new Date().toISOString();
    return (
      db.data.cycles.find(
        (cycle) =>
          cycle.teamId === team.id &&
          cycle.startsAt <= now &&
          now < cycle.endsAt,
      ) ?? null
    );
  },
  issues(team, args, { db }) {
    return issueConnection(
      db,
      db.data.issues.filter((issue) => issue.teamId === team.id),
      args,
    );
  },
  issueCounts(team, _args, { db }) {
    const counts = new Map<string, number>();
    for (const issue of db.data.issues) {
      if (issue.teamId === team.id && issue.archivedAt === null) {
        const type = db.stateTypeOf(issue);
        counts.set(type, (counts.get(type) ?? 0) + 1);
      }
    }
    return (
      [
        "TRIAGE",
        "BACKLOG",
        "UNSTARTED",
        "STARTED",
        "COMPLETED",
        "CANCELED",
      ] as const
    ).map((stateType) => ({ stateType, count: counts.get(stateType) ?? 0 }));
  },
};

export const TeamMembership: R["TeamMembership"] = {
  user: (membership, _args, { db }) => db.get("users", membership.userId),
  role: (membership) => membership.role,
  joinedAt: (membership) => membership.joinedAt,
};

export const WorkflowState: R["WorkflowState"] = {
  team(state, _args, { db }) {
    return db.get("teams", db.get("workflowStates", state.id).teamId);
  },
};

export const Label: R["Label"] = {
  team(label, _args, { db }) {
    return db.get("teams", db.get("labels", label.id).teamId);
  },
  openIssueCount(label, _args, { db }) {
    return db.data.issues.filter((issue) => {
      const type = db.stateTypeOf(issue);
      return (
        issue.labelIds.includes(label.id) &&
        type !== "COMPLETED" &&
        type !== "CANCELED"
      );
    }).length;
  },
};

export const Project: R["Project"] = {
  team(project, _args, { db }) {
    return db.get("teams", db.get("projects", project.id).teamId);
  },
  lead(project, _args, { db }) {
    const { leadId } = db.get("projects", project.id);
    return leadId === null ? null : db.get("users", leadId);
  },
  members(project, _args, { db }) {
    return db
      .get("projects", project.id)
      .memberIds.map((id) => db.get("users", id));
  },
  progress(project, _args, { db }) {
    return progressOf(
      db,
      db.data.issues.filter((issue) => issue.projectId === project.id),
    );
  },
  health(project, _args, { db }) {
    const updates = db.data.projectUpdates.filter(
      (update) => update.projectId === project.id,
    );
    updates.sort((a, b) => b.createdAt.localeCompare(a.createdAt));
    return updates[0]?.health ?? null;
  },
  milestones(project, _args, { db }) {
    return db.data.milestones.filter(
      (milestone) => milestone.projectId === project.id,
    );
  },
  issues(project, args, { db }) {
    return issueConnection(
      db,
      db.data.issues.filter((issue) => issue.projectId === project.id),
      args,
    );
  },
  updates(project, { first }, { db }) {
    return db.data.projectUpdates
      .filter((update) => update.projectId === project.id)
      .sort((a, b) => b.createdAt.localeCompare(a.createdAt))
      .slice(0, first ?? 10);
  },
};

export const Milestone: R["Milestone"] = {
  project(milestone, _args, { db }) {
    return db.get("projects", db.get("milestones", milestone.id).projectId);
  },
  progress(milestone, _args, { db }) {
    return progressOf(
      db,
      db.data.issues.filter((issue) => issue.milestoneId === milestone.id),
    );
  },
  issues(milestone, _args, { db }) {
    return db.data.issues.filter((issue) => issue.milestoneId === milestone.id);
  },
};

export const ProjectUpdate: R["ProjectUpdate"] = {
  author(update, _args, { db }) {
    return db.get("users", db.get("projectUpdates", update.id).authorId);
  },
  project(update, _args, { db }) {
    return db.get("projects", db.get("projectUpdates", update.id).projectId);
  },
  reactions(update, _args, { db }) {
    return reactionGroups(db, update.id);
  },
};

export const Cycle: R["Cycle"] = {
  team(cycle, _args, { db }) {
    return db.get("teams", db.get("cycles", cycle.id).teamId);
  },
  issues(cycle, args, { db }) {
    return issueConnection(
      db,
      db.data.issues.filter((issue) => issue.cycleId === cycle.id),
      args,
    );
  },
  progress(cycle, _args, { db }) {
    return progressOf(
      db,
      db.data.issues.filter((issue) => issue.cycleId === cycle.id),
    );
  },
  scope(cycle, _args, { db }) {
    return db.data.issues
      .filter((issue) => issue.cycleId === cycle.id)
      .reduce((sum, issue) => sum + (issue.estimate ?? 0), 0);
  },
  completedScope(cycle, _args, { db }) {
    return db.data.issues
      .filter(
        (issue) =>
          issue.cycleId === cycle.id && db.stateTypeOf(issue) === "COMPLETED",
      )
      .reduce((sum, issue) => sum + (issue.estimate ?? 0), 0);
  },
};

export const Issue: R["Issue"] = {
  identifier(issue, _args, { db }) {
    return db.identifierOf(db.get("issues", issue.id));
  },
  team(issue, _args, { db }) {
    return db.get("teams", db.get("issues", issue.id).teamId);
  },
  state(issue, _args, { db }) {
    return db.get("workflowStates", db.get("issues", issue.id).stateId);
  },
  assignee(issue, _args, { db }) {
    const { assigneeId } = db.get("issues", issue.id);
    return assigneeId === null ? null : db.get("users", assigneeId);
  },
  creator(issue, _args, { db }) {
    return db.get("users", db.get("issues", issue.id).creatorId);
  },
  labels(issue, _args, { db }) {
    return db
      .get("issues", issue.id)
      .labelIds.map((id) => db.get("labels", id));
  },
  project(issue, _args, { db }) {
    const { projectId } = db.get("issues", issue.id);
    return projectId === null ? null : db.get("projects", projectId);
  },
  milestone(issue, _args, { db }) {
    const { milestoneId } = db.get("issues", issue.id);
    return milestoneId === null ? null : db.get("milestones", milestoneId);
  },
  cycle(issue, _args, { db }) {
    const { cycleId } = db.get("issues", issue.id);
    return cycleId === null ? null : db.get("cycles", cycleId);
  },
  parent(issue, _args, { db }) {
    const { parentId } = db.get("issues", issue.id);
    return parentId === null ? null : db.get("issues", parentId);
  },
  children(issue, _args, { db }) {
    return db.data.issues.filter((child) => child.parentId === issue.id);
  },
  relations(issue, _args, { db }) {
    return db.data.relations.filter(
      (relation) => relation.issueId === issue.id,
    );
  },
  attachments(issue, _args, { db }) {
    return db.data.attachments.filter(
      (attachment) => attachment.issueId === issue.id,
    );
  },
  subscribers(issue, _args, { db }) {
    return db
      .get("issues", issue.id)
      .subscriberIds.map((id) => db.get("users", id));
  },
  viewerIsSubscribed(issue, _args, { db, viewerId }) {
    return db.get("issues", issue.id).subscriberIds.includes(viewerId);
  },
  comments(issue, { first, after }, { db }) {
    return paginate(
      db.data.comments.filter((comment) => comment.issueId === issue.id),
      first,
      after,
    );
  },
  timeline(issue, _args, { db }) {
    return db.timelineOf(issue.id);
  },
  reactions(issue, _args, { db }) {
    return reactionGroups(db, issue.id);
  },
};

export const IssueRelation: R["IssueRelation"] = {
  relatedIssue(relation, _args, { db }) {
    return db.get("issues", db.get("relations", relation.id).relatedIssueId);
  },
};

export const Attachment: R["Attachment"] = {
  creator(attachment, _args, { db }) {
    return db.get("users", db.get("attachments", attachment.id).creatorId);
  },
};

export const Comment: R["Comment"] = {
  author(comment, _args, { db }) {
    return db.get("users", db.get("comments", comment.id).authorId);
  },
  issue(comment, _args, { db }) {
    return db.get("issues", db.get("comments", comment.id).issueId);
  },
  parent(comment, _args, { db }) {
    const { parentId } = db.get("comments", comment.id);
    return parentId === null ? null : db.get("comments", parentId);
  },
  replies(comment, _args, { db }) {
    return db.data.comments.filter((reply) => reply.parentId === comment.id);
  },
  reactions(comment, _args, { db }) {
    return reactionGroups(db, comment.id);
  },
  viewerCanEdit(comment, _args, { db, viewerId }) {
    return db.get("comments", comment.id).authorId === viewerId;
  },
};

export const ReactionGroup: R["ReactionGroup"] = {
  emoji: (group) => group.emoji,
  count(group, _args, { db }) {
    return reactionUsers(db, group).length;
  },
  viewerHasReacted(group, _args, { db, viewerId }) {
    return reactionUsers(db, group).includes(viewerId);
  },
  users(group, { first }, { db }) {
    return reactionUsers(db, group)
      .slice(0, first ?? 5)
      .map((id) => db.get("users", id));
  },
};

export const IssueConnection: R["IssueConnection"] = {
  nodes: (connection) => connection.nodes,
  pageInfo: (connection) => connection.pageInfo,
  totalCount: (connection) => connection.totalCount,
};

export const CommentConnection: R["CommentConnection"] = {
  nodes: (connection) => connection.nodes,
  pageInfo: (connection) => connection.pageInfo,
  totalCount: (connection) => connection.totalCount,
};

export const NotificationConnection: R["NotificationConnection"] = {
  nodes: (connection) => connection.nodes,
  pageInfo: (connection) => connection.pageInfo,
  totalCount: (connection) => connection.totalCount,
  unreadCount: (connection) => connection.unreadCount,
};
