import { UserChip } from "@/components/UserChip";
import { ActivityIssueLink } from "../ActivityIssueLink";
import type { AssigneeChangedActivityFragment } from "./AssigneeChangedActivityFragment.graphql";

export const AssigneeChangedActivity: React.FC<{
  event: AssigneeChangedActivityFragment;
}> = ({ event }) => {
  const { actor, fromAssignee, toAssignee } = event;
  const issue = <ActivityIssueLink issue={event.issue} />;
  if (toAssignee === null) {
    return (
      <>
        <UserChip user={actor} /> unassigned{" "}
        {fromAssignee !== null ? <UserChip user={fromAssignee} /> : null} from{" "}
        {issue}
      </>
    );
  }
  if (toAssignee.id === actor.id) {
    return (
      <>
        <UserChip user={actor} /> self-assigned {issue}
      </>
    );
  }
  return (
    <>
      <UserChip user={actor} /> assigned {issue} to{" "}
      <UserChip user={toAssignee} />
    </>
  );
};
