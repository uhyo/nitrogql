import type {
  AttachmentKind,
  AttachmentRecord,
  CommentRecord,
  CycleRecord,
  IssueEventRecord,
  IssuePriority,
  IssueRecord,
  IssueRelationRecord,
  IssueRelationType,
  LabelRecord,
  MilestoneRecord,
  NotificationRecord,
  ProjectHealth,
  ProjectRecord,
  ProjectStatus,
  ProjectUpdateRecord,
  ReactionsRecord,
  TeamMembershipRecord,
  TeamRecord,
  TeamRole,
  UserRecord,
  WorkflowStateRecord,
  WorkflowStateType,
} from "./records";

/**
 * Generates deterministic sample data.
 * Dates are relative to `now`, so that the data looks fresh.
 */
export function seed(now: Date) {
  const random = mulberry32(20240501);
  const pick = <T>(items: readonly T[]): T =>
    items[Math.floor(random() * items.length)];
  const pickSome = <T>(items: readonly T[], max: number): T[] => {
    const count = Math.floor(random() * (max + 1));
    const pool = [...items];
    const result: T[] = [];
    for (let i = 0; i < count && pool.length > 0; i++) {
      result.push(pool.splice(Math.floor(random() * pool.length), 1)[0]);
    }
    return result;
  };
  const day = 24 * 60 * 60 * 1000;
  const daysAgo = (days: number) =>
    new Date(now.getTime() - days * day).toISOString();
  const dateOnly = (iso: string) => iso.slice(0, 10);
  const between = (from: string, to: string) => {
    const start = new Date(from).getTime();
    const end = new Date(to).getTime();
    return new Date(start + random() * (end - start)).toISOString();
  };

  const users: UserRecord[] = userSeeds.map(
    ([handle, name, timezone, bio], index) => ({
      id: `usr_${index + 1}`,
      handle,
      name,
      email: `${handle}@example.com`,
      avatarUrl:
        index % 4 === 3 ? null : `https://i.pravatar.cc/96?u=${handle}`,
      bio,
      timezone,
      deactivated: index === userSeeds.length - 1,
      status:
        index % 3 === 0
          ? {
              ...statusSeeds[index % statusSeeds.length],
              expiresAt: index % 2 === 0 ? daysAgo(-2) : null,
            }
          : null,
    }),
  );
  const activeUsers = users.filter((user) => !user.deactivated);

  const teams: TeamRecord[] = teamSeeds
    .map((team, index) => ({
      id: `team_${index + 1}`,
      ...team,
      createdAt: daysAgo(720 - index * 60),
      members: team.memberIndices.map(
        (userIndex, i): TeamMembershipRecord => ({
          userId: users[userIndex].id,
          role: (i === 0
            ? "OWNER"
            : i < 3
              ? "ADMIN"
              : i === team.memberIndices.length - 1
                ? "GUEST"
                : "MEMBER") satisfies TeamRole,
          joinedAt: daysAgo(700 - i * 37),
        }),
      ),
    }))
    .map(({ memberIndices: _, ...team }) => team);

  const workflowStates: WorkflowStateRecord[] = [];
  const labels: LabelRecord[] = [];
  for (const team of teams) {
    workflowStateSeeds.forEach(([name, type, color], position) => {
      workflowStates.push({
        id: `ws_${workflowStates.length + 1}`,
        teamId: team.id,
        name,
        type,
        color,
        position,
      });
    });
    for (const [name, color, description] of labelSeeds[team.key]) {
      labels.push({
        id: `lbl_${labels.length + 1}`,
        teamId: team.id,
        name,
        color,
        description,
      });
    }
  }
  const statesOf = (teamId: string) =>
    workflowStates.filter((state) => state.teamId === teamId);
  const membersOf = (teamId: string) =>
    teams
      .find((team) => team.id === teamId)!
      .members.map((member) => users.find((user) => user.id === member.userId)!)
      .filter((user) => !user.deactivated);

  const projects: ProjectRecord[] = [];
  const milestones: MilestoneRecord[] = [];
  const projectUpdates: ProjectUpdateRecord[] = [];
  for (const projectSeed of projectSeeds) {
    const team = teams.find((team) => team.key === projectSeed.teamKey)!;
    const members = membersOf(team.id);
    const startDate = dateOnly(daysAgo(projectSeed.startedDaysAgo));
    const project: ProjectRecord = {
      id: `prj_${projects.length + 1}`,
      teamId: team.id,
      slug: projectSeed.slug,
      name: projectSeed.name,
      description: projectSeed.description,
      icon: projectSeed.icon,
      color: projectSeed.color,
      status: projectSeed.status,
      startDate: projectSeed.status === "PLANNED" ? null : startDate,
      targetDate: dateOnly(daysAgo(projectSeed.startedDaysAgo - 90)),
      createdAt: daysAgo(projectSeed.startedDaysAgo + 14),
      leadId: members[projects.length % members.length].id,
      memberIds: pickSome(members, 5).map((user) => user.id),
    };
    projects.push(project);
    projectSeed.milestones.forEach((name, index) => {
      milestones.push({
        id: `ms_${milestones.length + 1}`,
        projectId: project.id,
        name,
        description:
          index === 0 ? `First deliverable of ${project.name}.` : null,
        targetDate: dateOnly(
          daysAgo(projectSeed.startedDaysAgo - 30 * (index + 1)),
        ),
      });
    });
    if (project.status !== "PLANNED") {
      const updateCount = 1 + Math.floor(random() * 3);
      for (let i = 0; i < updateCount; i++) {
        projectUpdates.push({
          id: `pu_${projectUpdates.length + 1}`,
          projectId: project.id,
          authorId: project.leadId ?? pick(members).id,
          body: pick(projectUpdateBodies),
          health: pick<ProjectHealth>([
            "ON_TRACK",
            "ON_TRACK",
            "AT_RISK",
            "OFF_TRACK",
          ]),
          createdAt: daysAgo(i * 7 + 1 + random() * 3),
        });
      }
    }
  }

  const cycles: CycleRecord[] = [];
  for (const team of teams) {
    if (team.key === "DES") {
      continue;
    }
    for (let number = 1; number <= 6; number++) {
      const startsAt = daysAgo((6 - number) * 14 + 7);
      const endsAt = daysAgo((6 - number) * 14 - 7);
      cycles.push({
        id: `cyc_${cycles.length + 1}`,
        teamId: team.id,
        number,
        name: number % 2 === 0 ? `Cycle ${number}: ${pick(cycleNames)}` : null,
        startsAt,
        endsAt,
        completedAt: number < 6 ? endsAt : null,
      });
    }
  }

  const issues: IssueRecord[] = [];
  const events: IssueEventRecord[] = [];
  const comments: CommentRecord[] = [];
  const relations: IssueRelationRecord[] = [];
  const attachments: AttachmentRecord[] = [];
  const reactions = new Map<string, ReactionsRecord>();
  const addReactions = (subjectId: string) => {
    const emojis = pickSome(reactionEmojis, 3);
    if (emojis.length === 0) {
      return;
    }
    reactions.set(
      subjectId,
      new Map(
        emojis.map((emoji) => [
          emoji,
          pickSome(activeUsers, 4).map((user) => user.id),
        ]),
      ),
    );
  };
  for (const team of teams) {
    const states = statesOf(team.id);
    const teamLabels = labels.filter((label) => label.teamId === team.id);
    const teamProjects = projects.filter(
      (project) => project.teamId === team.id,
    );
    const teamCycles = cycles.filter((cycle) => cycle.teamId === team.id);
    const members = membersOf(team.id);
    const count = team.key === "ENG" ? 90 : 35;
    const teamIssues: IssueRecord[] = [];
    for (let number = 1; number <= count; number++) {
      const createdAt = daysAgo(
        ((count - number) / count) * 150 + random() * 3,
      );
      const state = pick(
        states.flatMap((state) =>
          Array<WorkflowStateRecord>(stateWeights[state.type]).fill(state),
        ),
      );
      const project = random() < 0.6 ? pick(teamProjects) : null;
      const projectMilestones = project
        ? milestones.filter((milestone) => milestone.projectId === project.id)
        : [];
      const cycle =
        teamCycles.length > 0 && random() < 0.55 ? pick(teamCycles) : null;
      const [verb, subject] = [pick(issueVerbs), pick(issueSubjects[team.key])];
      const creator = pick(members);
      const assignee =
        state.type === "TRIAGE" || random() < 0.15 ? null : pick(members);
      const updatedAt = between(createdAt, now.toISOString());
      const issue: IssueRecord = {
        id: `iss_${issues.length + 1}`,
        teamId: team.id,
        number,
        title: `${verb} ${subject}`,
        description:
          random() < 0.8
            ? `${pick(descriptionIntros)}\n\n${pick(descriptionDetails)}`
            : null,
        priority: pick<IssuePriority>([
          "NO_PRIORITY",
          "URGENT",
          "HIGH",
          "HIGH",
          "MEDIUM",
          "MEDIUM",
          "MEDIUM",
          "LOW",
          "LOW",
        ]),
        estimate: random() < 0.7 ? pick([1, 2, 3, 5, 8]) : null,
        dueDate:
          random() < 0.25 ? dateOnly(daysAgo(-30 + random() * 45)) : null,
        createdAt,
        updatedAt,
        completedAt: state.type === "COMPLETED" ? updatedAt : null,
        canceledAt: state.type === "CANCELED" ? updatedAt : null,
        archivedAt:
          state.type === "CANCELED" && random() < 0.3 ? updatedAt : null,
        stateId: state.id,
        assigneeId: assignee?.id ?? null,
        creatorId: creator.id,
        labelIds: pickSome(teamLabels, 3).map((label) => label.id),
        projectId: project?.id ?? null,
        milestoneId:
          projectMilestones.length > 0 && random() < 0.7
            ? pick(projectMilestones).id
            : null,
        cycleId: cycle?.id ?? null,
        parentId:
          teamIssues.length > 5 && random() < 0.12
            ? pick(teamIssues.slice(-15)).id
            : null,
        subscriberIds: [
          ...new Set([
            creator.id,
            ...(assignee ? [assignee.id] : []),
            ...pickSome(members, 2).map((user) => user.id),
          ]),
        ],
      };
      issues.push(issue);
      teamIssues.push(issue);
      if (random() < 0.3) {
        addReactions(issue.id);
      }

      // Timeline of the issue
      const timestamps = Array.from({ length: 6 }, () =>
        between(createdAt, updatedAt),
      ).sort();
      const event = (
        data: DistributiveOmit<IssueEventRecord, "id" | "issueId">,
      ) => {
        events.push({
          id: `evt_${events.length + 1}`,
          issueId: issue.id,
          ...data,
        } as IssueEventRecord);
      };
      event({
        __typename: "IssueCreatedEvent",
        actorId: creator.id,
        createdAt,
      });
      if (issue.labelIds.length > 0 && random() < 0.5) {
        event({
          __typename: "LabelsChangedEvent",
          actorId: creator.id,
          createdAt: timestamps[0],
          addedLabelIds: issue.labelIds.slice(0, 1),
          removedLabelIds: [],
        });
      }
      if (assignee) {
        event({
          __typename: "AssigneeChangedEvent",
          actorId: pick(members).id,
          createdAt: timestamps[1],
          fromAssigneeId: null,
          toAssigneeId: assignee.id,
        });
      }
      if (random() < 0.3) {
        event({
          __typename: "PriorityChangedEvent",
          actorId: pick(members).id,
          createdAt: timestamps[2],
          fromPriority: "NO_PRIORITY",
          toPriority: issue.priority,
        });
      }
      if (project && random() < 0.4) {
        event({
          __typename: "ProjectChangedEvent",
          actorId: pick(members).id,
          createdAt: timestamps[2],
          fromProjectId: null,
          toProjectId: project.id,
        });
      }
      const commentCount = Math.floor(random() * 5);
      for (let i = 0; i < commentCount; i++) {
        const author = pick(members);
        const createdAt = between(timestamps[1], timestamps[5]);
        const parent =
          i > 0 && random() < 0.3 ? comments[comments.length - 1] : null;
        const comment: CommentRecord = {
          id: `cmt_${comments.length + 1}`,
          issueId: issue.id,
          authorId: author.id,
          parentId: parent?.id ?? null,
          body: pick(commentBodies).replace(
            "@someone",
            `@${pick(members).handle}`,
          ),
          createdAt,
          editedAt: random() < 0.15 ? between(createdAt, updatedAt) : null,
        };
        comments.push(comment);
        event({
          __typename: "CommentedEvent",
          actorId: author.id,
          createdAt,
          commentId: comment.id,
        });
        if (random() < 0.4) {
          addReactions(comment.id);
        }
      }
      const progression = states
        .filter(
          (s) =>
            s.type !== "TRIAGE" &&
            s.type !== "CANCELED" &&
            s.position <= state.position,
        )
        .slice(-3);
      let previous = states.find((s) => s.type === "BACKLOG")!;
      progression.forEach((toState, i) => {
        if (toState.id === previous.id) {
          return;
        }
        event({
          __typename: "StateChangedEvent",
          actorId: assignee?.id ?? pick(members).id,
          createdAt: timestamps[3 + i] ?? updatedAt,
          fromStateId: previous.id,
          toStateId: toState.id,
        });
        previous = toState;
      });
      if (state.type === "CANCELED") {
        event({
          __typename: "StateChangedEvent",
          actorId: pick(members).id,
          createdAt: updatedAt,
          fromStateId: previous.id,
          toStateId: state.id,
        });
      }
      if (random() < 0.3) {
        const kind = pick<AttachmentKind>([
          "LINK",
          "PULL_REQUEST",
          "PULL_REQUEST",
          "DESIGN",
          "DOCUMENT",
        ]);
        attachments.push({
          id: `att_${attachments.length + 1}`,
          issueId: issue.id,
          creatorId: pick(members).id,
          title: attachmentTitles[kind],
          url:
            kind === "PULL_REQUEST"
              ? `https://github.com/example/app/pull/${100 + attachments.length}`
              : `https://example.com/${kind.toLowerCase()}/${attachments.length + 1}`,
          kind,
          createdAt: timestamps[3],
        });
      }
    }
    for (const issue of teamIssues) {
      if (random() < 0.15) {
        const related = pick(teamIssues);
        if (related.id !== issue.id) {
          const type = pick<IssueRelationType>([
            "BLOCKS",
            "RELATED",
            "RELATED",
            "DUPLICATE_OF",
          ]);
          relations.push({
            id: `rel_${relations.length + 1}`,
            issueId: issue.id,
            relatedIssueId: related.id,
            type,
          });
          if (type === "BLOCKS") {
            relations.push({
              id: `rel_${relations.length + 1}`,
              issueId: related.id,
              relatedIssueId: issue.id,
              type: "BLOCKED_BY",
            });
          }
        }
      }
    }
  }
  for (const update of projectUpdates) {
    if (random() < 0.6) {
      addReactions(update.id);
    }
  }

  // Notifications of the viewer
  const viewer = users[0];
  const notifications: NotificationRecord[] = [];
  const notification = (
    data: DistributiveOmit<NotificationRecord, "id" | "userId" | "readAt">,
  ) => {
    notifications.push({
      id: `ntf_${notifications.length + 1}`,
      userId: viewer.id,
      readAt: random() < 0.4 ? data.createdAt : null,
      ...data,
    } as NotificationRecord);
  };
  for (const issue of issues
    .filter((issue) => issue.assigneeId === viewer.id)
    .slice(-12)) {
    notification({
      __typename: "IssueAssignedNotification",
      actorId: pick(activeUsers.filter((user) => user.id !== viewer.id)).id,
      createdAt: issue.createdAt,
      issueId: issue.id,
    });
    if (random() < 0.4) {
      notification({
        __typename: "IssueStateChangedNotification",
        actorId: pick(activeUsers.filter((user) => user.id !== viewer.id)).id,
        createdAt: issue.updatedAt,
        issueId: issue.id,
        stateId: issue.stateId,
      });
    }
  }
  for (const comment of comments
    .filter((comment) => comment.body.includes(`@${viewer.handle}`))
    .slice(-8)) {
    notification({
      __typename: "CommentMentionNotification",
      actorId: comment.authorId,
      createdAt: comment.createdAt,
      commentId: comment.id,
    });
  }
  const viewerCommentIds = new Set(
    comments
      .filter((comment) => comment.authorId === viewer.id)
      .map((comment) => comment.id),
  );
  for (const comment of comments
    .filter(
      (comment) =>
        comment.parentId !== null && viewerCommentIds.has(comment.parentId),
    )
    .slice(-6)) {
    notification({
      __typename: "CommentReplyNotification",
      actorId: comment.authorId,
      createdAt: comment.createdAt,
      commentId: comment.id,
    });
  }
  for (const update of projectUpdates.slice(-5)) {
    notification({
      __typename: "ProjectUpdateNotification",
      actorId: update.authorId,
      createdAt: update.createdAt,
      projectUpdateId: update.id,
    });
  }
  notifications.sort((a, b) => b.createdAt.localeCompare(a.createdAt));

  return {
    viewerId: viewer.id,
    users,
    teams,
    workflowStates,
    labels,
    projects,
    milestones,
    projectUpdates,
    cycles,
    issues,
    events,
    comments,
    relations,
    attachments,
    reactions,
    notifications,
  };
}

export type SeedData = ReturnType<typeof seed>;

type DistributiveOmit<T, K extends PropertyKey> = T extends unknown
  ? Omit<T, K>
  : never;

/** Small deterministic random number generator. */
function mulberry32(seed: number) {
  return () => {
    seed |= 0;
    seed = (seed + 0x6d2b79f5) | 0;
    let t = Math.imul(seed ^ (seed >>> 15), 1 | seed);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

const userSeeds: [
  handle: string,
  name: string,
  timezone: string,
  bio: string | null,
][] = [
  [
    "mika",
    "Mika Tanaka",
    "Asia/Tokyo",
    "Frontend engineer. Loves type systems.",
  ],
  ["oliver", "Oliver Brown", "Europe/London", "Backend and infrastructure."],
  ["amara", "Amara Okafor", "Africa/Lagos", "Product designer."],
  ["lucas", "Lucas Silva", "America/Sao_Paulo", null],
  ["sofia", "Sofia Rossi", "Europe/Rome", "Engineering manager."],
  ["noah", "Noah Kim", "Asia/Seoul", "Mobile and web."],
  ["priya", "Priya Patel", "Asia/Kolkata", "Data and analytics."],
  [
    "ethan",
    "Ethan Miller",
    "America/New_York",
    "SRE. On call more often than not.",
  ],
  ["lea", "Léa Martin", "Europe/Paris", "Design systems."],
  ["yusuf", "Yusuf Demir", "Europe/Istanbul", null],
  ["hana", "Hana Suzuki", "Asia/Tokyo", "QA and release management."],
  ["jack", "Jack Wilson", "Australia/Sydney", "Former platform lead."],
];

const statusSeeds = [
  { emoji: "🌴", message: "On vacation" },
  { emoji: "🎯", message: "Focusing" },
  { emoji: "🤒", message: "Out sick" },
  { emoji: "🚆", message: "Commuting" },
];

const teamSeeds = [
  {
    key: "ENG",
    name: "Engineering",
    description: "Builds and runs the product.",
    color: "#5e6ad2",
    icon: "🛠️",
    memberIndices: [4, 0, 1, 3, 5, 6, 7, 9, 10, 11],
  },
  {
    key: "DES",
    name: "Design",
    description: "Product and brand design.",
    color: "#f2994a",
    icon: "🎨",
    memberIndices: [2, 8, 0, 5],
  },
  {
    key: "OPS",
    name: "Operations",
    description: "Infrastructure, security and support.",
    color: "#26b5ce",
    icon: "📟",
    memberIndices: [7, 1, 9, 6, 10],
  },
];

const workflowStateSeeds: [string, WorkflowStateType, string][] = [
  ["Triage", "TRIAGE", "#fc7840"],
  ["Backlog", "BACKLOG", "#bec2c8"],
  ["Todo", "UNSTARTED", "#e2e2e2"],
  ["In Progress", "STARTED", "#f2c94c"],
  ["In Review", "STARTED", "#0f783c"],
  ["Done", "COMPLETED", "#5e6ad2"],
  ["Canceled", "CANCELED", "#95a2b3"],
];

const stateWeights: Record<WorkflowStateType, number> = {
  TRIAGE: 1,
  BACKLOG: 3,
  UNSTARTED: 3,
  STARTED: 3,
  COMPLETED: 5,
  CANCELED: 1,
};

const labelSeeds: Record<string, [string, string, string | null][]> = {
  ENG: [
    ["Bug", "#eb5757", "Something is not working."],
    ["Feature", "#bb87fc", "New functionality."],
    ["Improvement", "#4ea7fc", null],
    ["Performance", "#f2994a", "Make things faster."],
    ["Security", "#d63c3c", null],
    ["Tech debt", "#95a2b3", "Cleanups and refactoring."],
    ["Docs", "#26b5ce", null],
  ],
  DES: [
    ["UX", "#bb87fc", null],
    ["Visual", "#f2994a", null],
    ["Research", "#4cb782", "User research and interviews."],
    ["Accessibility", "#4ea7fc", null],
  ],
  OPS: [
    ["Incident", "#eb5757", "Follow-ups of incidents."],
    ["Monitoring", "#f2c94c", null],
    ["Cost", "#4cb782", null],
    ["Security", "#d63c3c", null],
  ],
};

const projectSeeds: {
  teamKey: string;
  slug: string;
  name: string;
  description: string | null;
  icon: string;
  color: string;
  status: ProjectStatus;
  startedDaysAgo: number;
  milestones: string[];
}[] = [
  {
    teamKey: "ENG",
    slug: "graphql-api-v2",
    name: "GraphQL API v2",
    description: "Replace the REST API with a typed GraphQL API.",
    icon: "🧬",
    color: "#e535ab",
    status: "IN_PROGRESS",
    startedDaysAgo: 80,
    milestones: ["Schema design", "Read API", "Mutations", "Deprecate REST"],
  },
  {
    teamKey: "ENG",
    slug: "mobile-app",
    name: "Mobile app",
    description: "Native apps for iOS and Android.",
    icon: "📱",
    color: "#4ea7fc",
    status: "IN_PROGRESS",
    startedDaysAgo: 60,
    milestones: ["Beta", "Public launch"],
  },
  {
    teamKey: "ENG",
    slug: "search-revamp",
    name: "Search revamp",
    description: "Faster and more relevant search.",
    icon: "🔎",
    color: "#f2c94c",
    status: "PAUSED",
    startedDaysAgo: 120,
    milestones: ["Indexing", "Ranking"],
  },
  {
    teamKey: "ENG",
    slug: "billing",
    name: "Billing",
    description: null,
    icon: "💳",
    color: "#4cb782",
    status: "COMPLETED",
    startedDaysAgo: 150,
    milestones: ["Invoices", "Usage-based pricing"],
  },
  {
    teamKey: "ENG",
    slug: "realtime-sync",
    name: "Realtime sync",
    description: "Live updates with subscriptions.",
    icon: "⚡",
    color: "#bb87fc",
    status: "PLANNED",
    startedDaysAgo: -10,
    milestones: [],
  },
  {
    teamKey: "DES",
    slug: "design-system",
    name: "Design system",
    description: "Shared components and tokens.",
    icon: "🧩",
    color: "#f2994a",
    status: "IN_PROGRESS",
    startedDaysAgo: 100,
    milestones: ["Tokens", "Components", "Documentation"],
  },
  {
    teamKey: "DES",
    slug: "onboarding",
    name: "Onboarding flow",
    description: "Help new users get started.",
    icon: "👋",
    color: "#26b5ce",
    status: "IN_PROGRESS",
    startedDaysAgo: 40,
    milestones: ["Research", "Prototype"],
  },
  {
    teamKey: "OPS",
    slug: "kubernetes-migration",
    name: "Kubernetes migration",
    description: "Move services to Kubernetes.",
    icon: "☸️",
    color: "#5e6ad2",
    status: "IN_PROGRESS",
    startedDaysAgo: 90,
    milestones: ["Staging", "Production", "Decommission VMs"],
  },
  {
    teamKey: "OPS",
    slug: "soc2",
    name: "SOC 2",
    description: "Get SOC 2 Type II certified.",
    icon: "🛡️",
    color: "#d63c3c",
    status: "CANCELED",
    startedDaysAgo: 140,
    milestones: ["Policies"],
  },
];

const cycleNames = ["Stabilization", "Polish", "Launch prep", "Performance"];

const issueVerbs = [
  "Fix",
  "Implement",
  "Refactor",
  "Investigate",
  "Add",
  "Remove",
  "Improve",
  "Document",
  "Migrate",
  "Redesign",
];

const issueSubjects: Record<string, string[]> = {
  ENG: [
    "login with SSO",
    "pagination of issue lists",
    "dark mode for settings",
    "slow dashboard query",
    "webhook retries",
    "N+1 queries in comments resolver",
    "rate limiting for public API",
    "file uploads over 100MB",
    "email notification preferences",
    "keyboard shortcuts",
    "markdown rendering of tables",
    "timezone handling in due dates",
    "offline support",
    "audit log export",
    "flaky end-to-end tests",
    "error tracking integration",
    "GraphQL schema documentation",
    "CSV import",
    "session expiration",
    "search highlighting",
  ],
  DES: [
    "empty states",
    "onboarding checklist",
    "icon set",
    "color contrast of labels",
    "settings page layout",
    "mobile navigation",
    "illustrations for errors",
    "typography scale",
  ],
  OPS: [
    "database backups",
    "alerting on error rates",
    "TLS certificate renewal",
    "log retention policy",
    "CI runner costs",
    "staging environment parity",
    "secrets rotation",
    "on-call runbook",
  ],
};

const descriptionIntros = [
  "Reported by several customers this week.",
  "Follow-up from the last planning meeting.",
  "This came up during the incident review.",
  "We need this before the next release.",
  "Small thing, but it bothers everyone.",
];

const descriptionDetails = [
  "## Steps to reproduce\n\n1. Open the page\n2. Click the button\n3. Nothing happens",
  "## Acceptance criteria\n\n- Works on mobile\n- Covered by tests\n- Documented",
  "See the attached design for details. Let's discuss open questions in comments.",
  "Proposal: start with the simplest version and iterate.",
];

const commentBodies = [
  "I can take a look at this tomorrow.",
  "Is this still happening on the latest version?",
  "@someone could you review the approach?",
  "Pushed a fix, waiting for CI.",
  "This is blocked by the API changes.",
  "Looks good to me 👍",
  "I think we should split this into smaller issues.",
  "Added a screenshot of the current state.",
  "@someone FYI, this affects the mobile app too.",
  "Moving this to the next cycle.",
];

const projectUpdateBodies = [
  "We are on track. The main work this week was finishing the read API.",
  "Some delays due to unexpected complexity; we will cut scope if needed.",
  "Blocked on the infrastructure work. Discussing alternatives.",
  "Shipped the first milestone 🎉 Next up: polishing and docs.",
];

const reactionEmojis = ["👍", "🎉", "❤️", "👀", "🚀", "😄"];

const attachmentTitles: Record<AttachmentKind, string> = {
  LINK: "Related discussion",
  PULL_REQUEST: "Pull request",
  DESIGN: "Figma design",
  DOCUMENT: "Specification",
};
