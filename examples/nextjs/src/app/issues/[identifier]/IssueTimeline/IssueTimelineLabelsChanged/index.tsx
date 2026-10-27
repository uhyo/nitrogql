import { LabelBadge } from "@/components/LabelBadge";
import { IssueTimelineEvent } from "../IssueTimelineEvent";
import type { IssueTimelineLabelsChangedFragment } from "./IssueTimelineLabelsChangedFragment.graphql";

export const IssueTimelineLabelsChanged: React.FC<{
  event: IssueTimelineLabelsChangedFragment;
}> = ({ event }) => (
  <IssueTimelineEvent event={event} icon="🏷">
    {event.addedLabels.length > 0 ? (
      <>
        <span className="muted">added</span>
        {event.addedLabels.map((label) => (
          <LabelBadge key={label.id} label={label} />
        ))}
      </>
    ) : null}
    {event.addedLabels.length > 0 && event.removedLabels.length > 0 ? (
      <span className="muted">and</span>
    ) : null}
    {event.removedLabels.length > 0 ? (
      <>
        <span className="muted">removed</span>
        {event.removedLabels.map((label) => (
          <LabelBadge key={label.id} label={label} />
        ))}
      </>
    ) : null}
  </IssueTimelineEvent>
);
