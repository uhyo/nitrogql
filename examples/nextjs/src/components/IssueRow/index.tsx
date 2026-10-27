import Link from "next/link";
import { formatDate } from "@/lib/format";
import { Avatar } from "../Avatar";
import { LabelBadge } from "../LabelBadge";
import { PriorityIcon } from "../PriorityIcon";
import { ProjectChip } from "../ProjectChip";
import { RelativeTime } from "../RelativeTime";
import { StateIcon } from "../StateIcon";
import type { IssueRowFragment } from "./IssueRowFragment.graphql";
import classes from "./IssueRow.module.css";

/** One line in a list of issues. */
export const IssueRow: React.FC<{
  issue: IssueRowFragment;
  /** Hides the project when all issues in the list belong to the same project. */
  hideProject?: boolean;
}> = ({ issue, hideProject = false }) => (
  <div className={classes.row}>
    <PriorityIcon priority={issue.priority} />
    <span className={`${classes.identifier} mono muted`}>
      {issue.identifier}
    </span>
    <StateIcon state={issue.state} />
    <Link href={`/issues/${issue.identifier}`} className={classes.title}>
      {issue.title}
    </Link>
    {issue.children.length > 0 ? (
      <span className="muted" title="Sub-issues">
        ⑂ {issue.children.length}
      </span>
    ) : null}
    <span className={classes.meta}>
      {issue.labels.map((label) => (
        <LabelBadge key={label.id} label={label} />
      ))}
      {!hideProject && issue.project !== null ? (
        <ProjectChip project={issue.project} />
      ) : null}
      {issue.dueDate !== null ? (
        <span className="muted" title="Due date">
          📅 {formatDate(issue.dueDate)}
        </span>
      ) : null}
      {issue.estimate !== null ? (
        <span className="muted" title="Estimate">
          {issue.estimate}pt
        </span>
      ) : null}
      <RelativeTime date={issue.updatedAt} className="muted" />
      {issue.assignee !== null ? (
        <Avatar user={issue.assignee} />
      ) : (
        <span className={classes.unassigned} title="Unassigned" />
      )}
    </span>
  </div>
);
