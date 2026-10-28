import { IssueTimelineEvent } from "../IssueTimelineEvent";
import type { IssueTimelineCreatedFragment } from "./IssueTimelineCreatedFragment.graphql";

export const IssueTimelineCreated: React.FC<{
  event: IssueTimelineCreatedFragment;
}> = ({ event }) => (
  <IssueTimelineEvent event={event} icon="✚">
    <span className="muted">created the issue</span>
  </IssueTimelineEvent>
);
