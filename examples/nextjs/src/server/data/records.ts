// Records stored in the in-memory database.
// Fields that are part of the GraphQL schema have the same names as in the schema;
// other fields hold references to other records by ID.

export type IssuePriority =
  | "NO_PRIORITY"
  | "URGENT"
  | "HIGH"
  | "MEDIUM"
  | "LOW";
export type WorkflowStateType =
  | "TRIAGE"
  | "BACKLOG"
  | "UNSTARTED"
  | "STARTED"
  | "COMPLETED"
  | "CANCELED";
export type ProjectStatus =
  | "PLANNED"
  | "IN_PROGRESS"
  | "PAUSED"
  | "COMPLETED"
  | "CANCELED";
export type ProjectHealth = "ON_TRACK" | "AT_RISK" | "OFF_TRACK";
export type TeamRole = "OWNER" | "ADMIN" | "MEMBER" | "GUEST";
export type IssueRelationType =
  | "BLOCKS"
  | "BLOCKED_BY"
  | "RELATED"
  | "DUPLICATE_OF";
export type AttachmentKind = "LINK" | "PULL_REQUEST" | "DESIGN" | "DOCUMENT";

export type UserRecord = {
  id: string;
  handle: string;
  name: string;
  email: string;
  avatarUrl: string | null;
  bio: string | null;
  timezone: string;
  deactivated: boolean;
  status: {
    emoji: string;
    message: string;
    expiresAt: string | null;
  } | null;
};

export type TeamRecord = {
  id: string;
  key: string;
  name: string;
  description: string | null;
  color: string;
  icon: string;
  createdAt: string;
  members: TeamMembershipRecord[];
};

export type TeamMembershipRecord = {
  userId: string;
  role: TeamRole;
  joinedAt: string;
};

export type WorkflowStateRecord = {
  id: string;
  teamId: string;
  name: string;
  type: WorkflowStateType;
  color: string;
  position: number;
};

export type LabelRecord = {
  id: string;
  teamId: string;
  name: string;
  color: string;
  description: string | null;
};

export type ProjectRecord = {
  id: string;
  teamId: string;
  slug: string;
  name: string;
  description: string | null;
  icon: string;
  color: string;
  status: ProjectStatus;
  startDate: string | null;
  targetDate: string | null;
  createdAt: string;
  leadId: string | null;
  memberIds: string[];
};

export type MilestoneRecord = {
  id: string;
  projectId: string;
  name: string;
  description: string | null;
  targetDate: string | null;
};

export type ProjectUpdateRecord = {
  id: string;
  projectId: string;
  authorId: string;
  body: string;
  health: ProjectHealth;
  createdAt: string;
};

export type CycleRecord = {
  id: string;
  teamId: string;
  number: number;
  name: string | null;
  startsAt: string;
  endsAt: string;
  completedAt: string | null;
};

export type IssueRecord = {
  id: string;
  teamId: string;
  number: number;
  title: string;
  description: string | null;
  priority: IssuePriority;
  estimate: number | null;
  dueDate: string | null;
  createdAt: string;
  updatedAt: string;
  completedAt: string | null;
  canceledAt: string | null;
  archivedAt: string | null;
  stateId: string;
  assigneeId: string | null;
  creatorId: string;
  labelIds: string[];
  projectId: string | null;
  milestoneId: string | null;
  cycleId: string | null;
  parentId: string | null;
  subscriberIds: string[];
};

export type IssueRelationRecord = {
  id: string;
  issueId: string;
  relatedIssueId: string;
  type: IssueRelationType;
};

export type AttachmentRecord = {
  id: string;
  issueId: string;
  creatorId: string;
  title: string;
  url: string;
  kind: AttachmentKind;
  createdAt: string;
};

export type CommentRecord = {
  id: string;
  issueId: string;
  authorId: string;
  parentId: string | null;
  body: string;
  createdAt: string;
  editedAt: string | null;
};

type EventBase = {
  id: string;
  issueId: string;
  actorId: string;
  createdAt: string;
};

export type IssueEventRecord = EventBase &
  (
    | { __typename: "IssueCreatedEvent" }
    | {
        __typename: "StateChangedEvent";
        fromStateId: string;
        toStateId: string;
      }
    | {
        __typename: "AssigneeChangedEvent";
        fromAssigneeId: string | null;
        toAssigneeId: string | null;
      }
    | {
        __typename: "PriorityChangedEvent";
        fromPriority: IssuePriority;
        toPriority: IssuePriority;
      }
    | {
        __typename: "LabelsChangedEvent";
        addedLabelIds: string[];
        removedLabelIds: string[];
      }
    | {
        __typename: "ProjectChangedEvent";
        fromProjectId: string | null;
        toProjectId: string | null;
      }
    | { __typename: "CommentedEvent"; commentId: string }
  );

type NotificationBase = {
  id: string;
  userId: string;
  actorId: string;
  createdAt: string;
  readAt: string | null;
};

export type NotificationRecord = NotificationBase &
  (
    | { __typename: "IssueAssignedNotification"; issueId: string }
    | {
        __typename: "IssueStateChangedNotification";
        issueId: string;
        stateId: string;
      }
    | { __typename: "CommentMentionNotification"; commentId: string }
    | { __typename: "CommentReplyNotification"; commentId: string }
    | { __typename: "ProjectUpdateNotification"; projectUpdateId: string }
  );

/**
 * Reactions to an issue, a comment or a project update:
 * emoji → IDs of users who reacted, in order of reaction.
 */
export type ReactionsRecord = Map<string, string[]>;

/** Model of a group of reactions with the same emoji. */
export type ReactionGroupModel = {
  subjectId: string;
  emoji: string;
};
