import type { Database } from "../data/db";
import { priorityOrder } from "../data/db";
import type { IssueRecord, ReactionGroupModel } from "../data/records";
import type * as Schema from "@/generated/schema";

type IssueFilter = Schema.__ResolverInput.IssueFilter;
type IssueSort = Schema.__ResolverInput.IssueSort;

/** Filters issues by a filter given by a client. */
export function filterIssues(
  db: Database,
  issues: readonly IssueRecord[],
  filter: IssueFilter | null | undefined,
): IssueRecord[] {
  const search = filter?.search?.toLowerCase();
  return issues.filter((issue) => {
    if (!filter?.includeArchived && issue.archivedAt !== null) {
      return false;
    }
    if (filter == null) {
      return true;
    }
    if (
      search &&
      !issue.title.toLowerCase().includes(search) &&
      !issue.description?.toLowerCase().includes(search) &&
      db.identifierOf(issue).toLowerCase() !== search
    ) {
      return false;
    }
    if (
      filter.stateTypes &&
      !filter.stateTypes.includes(db.stateTypeOf(issue))
    ) {
      return false;
    }
    if (
      filter.assigneeIds &&
      (issue.assigneeId === null ||
        !filter.assigneeIds.includes(issue.assigneeId))
    ) {
      return false;
    }
    if (filter.unassigned && issue.assigneeId !== null) {
      return false;
    }
    if (
      filter.labelIds &&
      !filter.labelIds.some((id) => issue.labelIds.includes(id))
    ) {
      return false;
    }
    if (filter.priorities && !filter.priorities.includes(issue.priority)) {
      return false;
    }
    if (filter.projectId && issue.projectId !== filter.projectId) {
      return false;
    }
    if (filter.cycleId && issue.cycleId !== filter.cycleId) {
      return false;
    }
    return true;
  });
}

/** Sorts issues. By default, the most recently updated issues come first. */
export function sortIssues(
  issues: IssueRecord[],
  sort: IssueSort | null | undefined,
): IssueRecord[] {
  const field = sort?.field ?? "UPDATED_AT";
  const direction = sort?.direction === "ASC" ? 1 : -1;
  const key = (issue: IssueRecord): string | number => {
    switch (field) {
      case "CREATED_AT":
        return issue.createdAt;
      case "UPDATED_AT":
        return issue.updatedAt;
      case "PRIORITY":
        // Higher priority is "greater"
        return -priorityOrder[issue.priority];
      case "DUE_DATE":
        return issue.dueDate ?? (direction > 0 ? "9999" : "");
      case "NUMBER":
        return issue.number;
    }
  };
  return issues.sort((a, b) => {
    const [ka, kb] = [key(a), key(b)];
    return (ka < kb ? -1 : ka > kb ? 1 : 0) * direction;
  });
}

/** Cursor-based pagination. Cursors are offsets. */
export function paginate<T>(
  items: readonly T[],
  first: number | null | undefined,
  after: string | null | undefined,
) {
  const start = after ? Number(after) : 0;
  const end = start + (first ?? 50);
  return {
    nodes: items.slice(start, end),
    pageInfo: {
      hasNextPage: end < items.length,
      endCursor: end < items.length ? String(end) : null,
    },
    totalCount: items.length,
  };
}

/** Builds an issue connection from issues. */
export function issueConnection(
  db: Database,
  issues: readonly IssueRecord[],
  args: {
    filter?: IssueFilter | null;
    sort?: IssueSort | null;
    first: number | null;
    after?: string | null;
  },
) {
  return paginate(
    sortIssues(filterIssues(db, issues, args.filter), args.sort),
    args.first,
    args.after,
  );
}

/** Ratio of completed estimate (counting issues without estimates as 1). */
export function progressOf(db: Database, issues: readonly IssueRecord[]) {
  const relevant = issues.filter(
    (issue) => db.stateTypeOf(issue) !== "CANCELED",
  );
  const total = relevant.reduce((sum, issue) => sum + (issue.estimate ?? 1), 0);
  const completed = relevant
    .filter((issue) => db.stateTypeOf(issue) === "COMPLETED")
    .reduce((sum, issue) => sum + (issue.estimate ?? 1), 0);
  return total === 0 ? 0 : completed / total;
}

/** Reaction groups of a subject. */
export function reactionGroups(
  db: Database,
  subjectId: string,
): ReactionGroupModel[] {
  return [...db.reactionsOf(subjectId)]
    .filter(([, userIds]) => userIds.length > 0)
    .map(([emoji]) => ({ emoji, subjectId }));
}
