import Link from "next/link";
import { UserChip } from "@/components/UserChip";
import { ActivityIssueLink } from "../ActivityIssueLink";
import type { CommentedActivityFragment } from "./CommentedActivityFragment.graphql";
import classes from "../RecentActivity.module.css";

const maxLength = 120;

export const CommentedActivity: React.FC<{
  event: CommentedActivityFragment;
}> = ({ event }) => {
  const body = event.comment.body.replace(/\s+/g, " ").trim();
  return (
    <>
      <UserChip user={event.actor} /> commented on{" "}
      <ActivityIssueLink issue={event.issue} />
      <Link
        href={`/issues/${event.issue.identifier}#comment-${event.comment.id}`}
        className={classes.quote}
      >
        {body.length > maxLength ? `${body.slice(0, maxLength)}…` : body}
      </Link>
    </>
  );
};
