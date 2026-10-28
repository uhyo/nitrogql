import { UserChip } from "@/components/UserChip";
import { IssueTimelineEvent } from "../IssueTimelineEvent";
import type { IssueTimelineAssigneeChangedFragment } from "./IssueTimelineAssigneeChangedFragment.graphql";

export const IssueTimelineAssigneeChanged: React.FC<{
  event: IssueTimelineAssigneeChangedFragment;
}> = ({ event }) => {
  const { fromAssignee, toAssignee } = event;
  return (
    <IssueTimelineEvent event={event} icon="👤">
      {toAssignee === null ? (
        fromAssignee !== null ? (
          <>
            <span className="muted">unassigned</span>
            <UserChip user={fromAssignee} />
          </>
        ) : (
          <span className="muted">removed the assignee</span>
        )
      ) : toAssignee.id === event.actor.id && fromAssignee === null ? (
        <span className="muted">self-assigned the issue</span>
      ) : (
        <>
          <span className="muted">assigned the issue to</span>
          <UserChip user={toAssignee} />
          {fromAssignee !== null ? (
            <>
              <span className="muted">instead of</span>
              <UserChip user={fromAssignee} />
            </>
          ) : null}
        </>
      )}
    </IssueTimelineEvent>
  );
};
