import { StateIcon } from "@/components/StateIcon";
import { InboxIssueLink } from "../InboxIssueLink";
import type { IssueStateChangedNotificationFragment } from "./IssueStateChangedNotificationFragment.graphql";

export const IssueStateChangedNotification: React.FC<{
  notification: IssueStateChangedNotificationFragment;
}> = ({ notification }) => (
  <>
    moved <InboxIssueLink issue={notification.issue} /> to{" "}
    <StateIcon state={notification.state} showName />
  </>
);
