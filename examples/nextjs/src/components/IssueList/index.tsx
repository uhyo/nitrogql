import { StateIcon } from "../StateIcon";
import { IssueRow } from "../IssueRow";
import type { IssueListFragment } from "./IssueListFragment.graphql";
import classes from "./IssueList.module.css";

/**
 * List of issues. With `groupByState`, issues are grouped by workflow states.
 * States with the same name in different teams are grouped together.
 */
export const IssueList: React.FC<{
  issues: IssueListFragment;
  groupByState?: boolean;
  hideProject?: boolean;
  emptyMessage?: string;
  onLoadMore?: (cursor: string) => void;
}> = ({
  issues,
  groupByState = false,
  hideProject,
  emptyMessage = "No issues",
  onLoadMore,
}) => {
  if (issues.nodes.length === 0) {
    return <p className={`${classes.empty} muted`}>{emptyMessage}</p>;
  }
  const groups = groupByState
    ? groupBy(issues.nodes, (issue) => issue.state)
    : [{ key: null, items: issues.nodes }];
  const { pageInfo } = issues;
  return (
    <div className={classes.list}>
      {groups.map(({ key: state, items }) => (
        <section key={state?.name ?? "all"}>
          {state !== null ? (
            <h3 className={classes.groupHeader}>
              <StateIcon state={{ ...state, color: "currentColor" }} />
              {state.name}
              <span className="muted">{items.length}</span>
            </h3>
          ) : null}
          {items.map((issue) => (
            <IssueRow key={issue.id} issue={issue} hideProject={hideProject} />
          ))}
        </section>
      ))}
      <footer className={`${classes.footer} muted`}>
        {issues.nodes.length} of {issues.totalCount} issues
        {onLoadMore && pageInfo.hasNextPage && pageInfo.endCursor !== null ? (
          <button
            type="button"
            className="button"
            onClick={() => onLoadMore(pageInfo.endCursor!)}
          >
            Load more
          </button>
        ) : null}
      </footer>
    </div>
  );
};

function groupBy<T, K extends { name: string; position: number }>(
  items: readonly T[],
  getKey: (item: T) => K,
): { key: K; items: T[] }[] {
  const groups = new Map<string, { key: K; items: T[] }>();
  for (const item of items) {
    const key = getKey(item);
    let group = groups.get(key.name);
    if (group === undefined) {
      group = { key, items: [] };
      groups.set(key.name, group);
    }
    group.items.push(item);
  }
  return [...groups.values()].sort((a, b) => a.key.position - b.key.position);
}
