import Link from "next/link";
import type { ActivityIssueLinkFragment } from "./ActivityIssueLinkFragment.graphql";
import classes from "../RecentActivity.module.css";

export const ActivityIssueLink: React.FC<{
  issue: ActivityIssueLinkFragment;
}> = ({ issue }) => (
  <Link href={`/issues/${issue.identifier}`} className={classes.issue}>
    <span className="mono muted">{issue.identifier}</span> {issue.title}
  </Link>
);
