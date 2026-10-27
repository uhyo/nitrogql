import Link from "next/link";
import type { InboxCommentExcerptFragment } from "./InboxCommentExcerptFragment.graphql";
import classes from "./InboxCommentExcerpt.module.css";

const maxLength = 160;

/** Beginning of a comment, linking to the comment in its issue. */
export const InboxCommentExcerpt: React.FC<{
  comment: InboxCommentExcerptFragment;
}> = ({ comment }) => (
  <Link
    href={`/issues/${comment.issue.identifier}#comment-${comment.id}`}
    className={classes.excerpt}
  >
    {excerpt(comment.body)}
  </Link>
);

/** Shortens Markdown text to a single line. */
export function excerpt(body: string, length = maxLength): string {
  const text = body.replace(/\s+/g, " ").trim();
  return text.length > length ? `${text.slice(0, length)}…` : text;
}
