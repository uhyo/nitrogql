import { InboxCommentExcerpt, excerpt } from "../InboxCommentExcerpt";
import { InboxIssueLink } from "../InboxIssueLink";
import type { CommentReplyNotificationFragment } from "./CommentReplyNotificationFragment.graphql";

export const CommentReplyNotification: React.FC<{
  notification: CommentReplyNotificationFragment;
}> = ({ notification: { comment } }) => (
  <>
    replied to{" "}
    {comment.parent !== null ? (
      <span title={comment.parent.body}>
        “{excerpt(comment.parent.body, 40)}”
      </span>
    ) : (
      "your comment"
    )}{" "}
    on <InboxIssueLink issue={comment.issue} />
    <InboxCommentExcerpt comment={comment} />
  </>
);
