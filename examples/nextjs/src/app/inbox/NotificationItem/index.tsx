"use client";

import { useMutation } from "urql";
import { RelativeTime } from "@/components/RelativeTime";
import { UserChip } from "@/components/UserChip";
import { CommentMentionNotification } from "./CommentMentionNotification";
import { CommentReplyNotification } from "./CommentReplyNotification";
import { IssueAssignedNotification } from "./IssueAssignedNotification";
import { IssueStateChangedNotification } from "./IssueStateChangedNotification";
import { ProjectUpdateNotification } from "./ProjectUpdateNotification";
import type { NotificationItemFragment } from "./NotificationItemFragment.graphql";
import MarkNotificationReadMutation from "./markNotificationRead.graphql";
import classes from "./NotificationItem.module.css";

/**
 * Typenames of queries to refetch after marking a notification.
 * The Sidebar only selects `unreadCount` of the connection, so the mutation
 * result (a `Notification`) alone would not invalidate it.
 */
const refetchTypenames = ["NotificationConnection"];

/** One notification in the inbox, with a button to mark it read or unread. */
export const NotificationItem: React.FC<{
  notification: NotificationItemFragment;
}> = ({ notification }) => {
  const [{ fetching }, markRead] = useMutation(MarkNotificationReadMutation);
  const unread = notification.readAt === null;
  return (
    <li className={classes.item} data-unread={unread || undefined}>
      <span
        className={classes.dot}
        aria-label={unread ? "Unread" : undefined}
      />
      <div className={classes.body}>
        <UserChip user={notification.actor} />{" "}
        <NotificationBody notification={notification} />
      </div>
      <RelativeTime
        date={notification.createdAt}
        className={`${classes.time} muted`}
      />
      <button
        type="button"
        className={`button ${classes.action}`}
        disabled={fetching}
        onClick={() =>
          markRead(
            { id: notification.id, read: unread },
            { additionalTypenames: refetchTypenames },
          )
        }
      >
        {unread ? "Mark read" : "Mark unread"}
      </button>
    </li>
  );
};

const NotificationBody: React.FC<{
  notification: NotificationItemFragment;
}> = ({ notification }) => {
  switch (notification.__typename) {
    case "IssueAssignedNotification":
      return <IssueAssignedNotification notification={notification} />;
    case "IssueStateChangedNotification":
      return <IssueStateChangedNotification notification={notification} />;
    case "CommentMentionNotification":
      return <CommentMentionNotification notification={notification} />;
    case "CommentReplyNotification":
      return <CommentReplyNotification notification={notification} />;
    case "ProjectUpdateNotification":
      return <ProjectUpdateNotification notification={notification} />;
  }
};
