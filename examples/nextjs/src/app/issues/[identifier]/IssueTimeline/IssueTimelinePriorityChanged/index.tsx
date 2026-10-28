import { PriorityIcon } from "@/components/PriorityIcon";
import { IssueTimelineEvent } from "../IssueTimelineEvent";
import type { IssueTimelinePriorityChangedFragment } from "./IssueTimelinePriorityChangedFragment.graphql";

export const IssueTimelinePriorityChanged: React.FC<{
  event: IssueTimelinePriorityChangedFragment;
}> = ({ event }) => (
  <IssueTimelineEvent
    event={event}
    icon={<PriorityIcon priority={event.toPriority} />}
  >
    <span className="muted">changed the priority from</span>
    <PriorityIcon priority={event.fromPriority} showName />
    <span className="muted">to</span>
    <PriorityIcon priority={event.toPriority} showName />
  </IssueTimelineEvent>
);
