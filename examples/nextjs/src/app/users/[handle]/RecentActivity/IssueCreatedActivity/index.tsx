import { UserChip } from "@/components/UserChip";
import { ActivityIssueLink } from "../ActivityIssueLink";
import type { IssueCreatedActivityFragment } from "./IssueCreatedActivityFragment.graphql";

export const IssueCreatedActivity: React.FC<{
  event: IssueCreatedActivityFragment;
}> = ({ event }) => (
  <>
    <UserChip user={event.actor} /> created{" "}
    <ActivityIssueLink issue={event.issue} />
  </>
);
