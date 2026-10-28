import Link from "next/link";
import { StateIcon } from "@/components/StateIcon";
import type { InboxIssueLinkFragment } from "./InboxIssueLinkFragment.graphql";
import classes from "./InboxIssueLink.module.css";

/** Link to an issue with its state, identifier and title. */
export const InboxIssueLink: React.FC<{ issue: InboxIssueLinkFragment }> = ({
  issue,
}) => (
  <Link href={`/issues/${issue.identifier}`} className={classes.link}>
    <StateIcon state={issue.state} />
    <span className="mono muted">{issue.identifier}</span>
    <span className={classes.title}>{issue.title}</span>
  </Link>
);
