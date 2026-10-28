import { ProgressBar } from "@/components/ProgressBar";
import { UserChip } from "@/components/UserChip";
import type { CycleAssigneeBreakdownFragment } from "./CycleAssigneeBreakdownFragment.graphql";
import classes from "./CycleAssigneeBreakdown.module.css";

type Assignee = NonNullable<
  CycleAssigneeBreakdownFragment["nodes"][number]["assignee"]
>;

type Row = {
  assignee: Assignee | null;
  issues: number;
  completedIssues: number;
  scope: number;
  completedScope: number;
};

/** Issues and scope of a cycle per assignee. Canceled issues are not counted. */
export const CycleAssigneeBreakdown: React.FC<{
  issues: CycleAssigneeBreakdownFragment;
}> = ({ issues }) => {
  const rows = breakdown(issues.nodes);
  if (rows.length === 0) {
    return <p className="muted">No issues in this cycle.</p>;
  }
  return (
    <table className={classes.table}>
      <thead>
        <tr>
          <th>Assignee</th>
          <th>Issues</th>
          <th>Scope</th>
          <th>Progress</th>
        </tr>
      </thead>
      <tbody>
        {rows.map((row) => (
          <tr key={row.assignee?.id ?? "unassigned"}>
            <td>
              {row.assignee !== null ? (
                <UserChip user={row.assignee} />
              ) : (
                <span className="muted">Unassigned</span>
              )}
            </td>
            <td>
              {row.completedIssues} / {row.issues}
            </td>
            <td>
              {row.completedScope} / {row.scope} pt
            </td>
            <td>
              <ProgressBar
                progress={
                  row.issues === 0 ? 0 : row.completedIssues / row.issues
                }
              />
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
};

function breakdown(issues: CycleAssigneeBreakdownFragment["nodes"]): Row[] {
  const rows = new Map<string | null, Row>();
  for (const issue of issues) {
    if (issue.state.type === "CANCELED") {
      continue;
    }
    const key = issue.assignee?.id ?? null;
    let row = rows.get(key);
    if (row === undefined) {
      row = {
        assignee: issue.assignee,
        issues: 0,
        completedIssues: 0,
        scope: 0,
        completedScope: 0,
      };
      rows.set(key, row);
    }
    const completed = issue.state.type === "COMPLETED";
    const estimate = issue.estimate ?? 0;
    row.issues++;
    row.scope += estimate;
    if (completed) {
      row.completedIssues++;
      row.completedScope += estimate;
    }
  }
  // Most loaded first, unassigned last
  return [...rows.values()].sort(
    (a, b) =>
      Number(a.assignee === null) - Number(b.assignee === null) ||
      b.scope - a.scope ||
      b.issues - a.issues,
  );
}
