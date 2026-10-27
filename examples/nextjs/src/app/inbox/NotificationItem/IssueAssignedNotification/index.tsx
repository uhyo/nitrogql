import { InboxIssueLink } from "../InboxIssueLink";
import type { IssueAssignedNotificationFragment } from "./IssueAssignedNotificationFragment.graphql";

export const IssueAssignedNotification: React.FC<{
  notification: IssueAssignedNotificationFragment;
}> = ({ notification }) => (
  <>
    assigned you to <InboxIssueLink issue={notification.issue} />
  </>
);
