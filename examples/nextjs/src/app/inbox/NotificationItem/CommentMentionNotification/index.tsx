import { InboxCommentExcerpt } from "../InboxCommentExcerpt";
import { InboxIssueLink } from "../InboxIssueLink";
import type { CommentMentionNotificationFragment } from "./CommentMentionNotificationFragment.graphql";

export const CommentMentionNotification: React.FC<{
  notification: CommentMentionNotificationFragment;
}> = ({ notification: { comment } }) => (
  <>
    mentioned you in a comment on <InboxIssueLink issue={comment.issue} />
    <InboxCommentExcerpt comment={comment} />
  </>
);
