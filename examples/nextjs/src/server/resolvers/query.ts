import type { ResolverOutput, Resolvers } from "@/generated/resolvers";
import type { Context } from "./context";
import { paginate } from "./helpers";

export const Query: Resolvers<Context>["Query"] = {
  viewer(_parent, _args, { db, viewerId }) {
    return db.get("users", viewerId);
  },
  node(_parent, { id }, { db }) {
    // Every record in the database is a Node.
    return (db.findAny(id)?.record ?? null) as ResolverOutput<"Node"> | null;
  },
  user(_parent, { handle }, { db }) {
    return db.data.users.find((user) => user.handle === handle) ?? null;
  },
  users(_parent, { includeDeactivated }, { db }) {
    return db.data.users.filter(
      (user) => includeDeactivated || !user.deactivated,
    );
  },
  team(_parent, { key }, { db }) {
    return db.data.teams.find((team) => team.key === key.toUpperCase()) ?? null;
  },
  teams(_parent, _args, { db }) {
    return db.data.teams;
  },
  issue(_parent, { identifier }, { db }) {
    return db.findIssueByIdentifier(identifier) ?? null;
  },
  project(_parent, { slug }, { db }) {
    return db.data.projects.find((project) => project.slug === slug) ?? null;
  },
  projects(_parent, { status, teamKey }, { db }) {
    const team = teamKey
      ? db.data.teams.find((team) => team.key === teamKey.toUpperCase())
      : undefined;
    return db.data.projects.filter(
      (project) =>
        (status == null || status.includes(project.status)) &&
        (teamKey == null || project.teamId === team?.id),
    );
  },
  cycle(_parent, { teamKey, number }, { db }) {
    const team = db.data.teams.find(
      (team) => team.key === teamKey.toUpperCase(),
    );
    return (
      db.data.cycles.find(
        (cycle) => cycle.teamId === team?.id && cycle.number === number,
      ) ?? null
    );
  },
  notifications(_parent, { unreadOnly, first, after }, { db, viewerId }) {
    const notifications = db.data.notifications.filter(
      (notification) => notification.userId === viewerId,
    );
    return {
      ...paginate(
        notifications.filter(
          (notification) => !unreadOnly || notification.readAt === null,
        ),
        first,
        after,
      ),
      unreadCount: notifications.filter(
        (notification) => notification.readAt === null,
      ).length,
    };
  },
  search(_parent, { query, kinds, first }, { db }) {
    const q = query.trim().toLowerCase();
    if (q === "") {
      return [];
    }
    const includes = (kind: "ISSUE" | "PROJECT" | "USER" | "TEAM") =>
      kinds == null || kinds.includes(kind);
    const results = [
      ...(includes("TEAM")
        ? db.data.teams.filter(
            (team) =>
              team.name.toLowerCase().includes(q) ||
              team.key.toLowerCase() === q,
          )
        : []),
      ...(includes("PROJECT")
        ? db.data.projects.filter((project) =>
            project.name.toLowerCase().includes(q),
          )
        : []),
      ...(includes("USER")
        ? db.data.users.filter(
            (user) =>
              user.name.toLowerCase().includes(q) ||
              user.handle.toLowerCase().includes(q),
          )
        : []),
      ...(includes("ISSUE")
        ? db.data.issues.filter(
            (issue) =>
              issue.title.toLowerCase().includes(q) ||
              db.identifierOf(issue).toLowerCase() === q,
          )
        : []),
    ];
    return results.slice(0, first ?? 20);
  },
};
