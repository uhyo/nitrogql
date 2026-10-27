import { StateIcon } from "@/components/StateIcon";
import { UserChip } from "@/components/UserChip";
import { ActivityIssueLink } from "../ActivityIssueLink";
import type { StateChangedActivityFragment } from "./StateChangedActivityFragment.graphql";

export const StateChangedActivity: React.FC<{
  event: StateChangedActivityFragment;
}> = ({ event }) => (
  <>
    <UserChip user={event.actor} /> moved{" "}
    <ActivityIssueLink issue={event.issue} /> from{" "}
    <StateIcon state={event.fromState} showName /> to{" "}
    <StateIcon state={event.toState} showName />
  </>
);
