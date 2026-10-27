import { StateIcon } from "@/components/StateIcon";
import { IssueTimelineEvent } from "../IssueTimelineEvent";
import type { IssueTimelineStateChangedFragment } from "./IssueTimelineStateChangedFragment.graphql";

export const IssueTimelineStateChanged: React.FC<{
  event: IssueTimelineStateChangedFragment;
}> = ({ event }) => (
  <IssueTimelineEvent event={event} icon={<StateIcon state={event.toState} />}>
    <span className="muted">changed the state from</span>
    <StateIcon state={event.fromState} showName />
    <span className="muted">to</span>
    <StateIcon state={event.toState} showName />
  </IssueTimelineEvent>
);
