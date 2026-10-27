import { PriorityIcon } from "@/components/PriorityIcon";
import { UserChip } from "@/components/UserChip";
import { ActivityIssueLink } from "../ActivityIssueLink";
import type { PriorityChangedActivityFragment } from "./PriorityChangedActivityFragment.graphql";

export const PriorityChangedActivity: React.FC<{
  event: PriorityChangedActivityFragment;
}> = ({ event }) => (
  <>
    <UserChip user={event.actor} /> changed the priority of{" "}
    <ActivityIssueLink issue={event.issue} /> from{" "}
    <PriorityIcon priority={event.fromPriority} showName /> to{" "}
    <PriorityIcon priority={event.toPriority} showName />
  </>
);
