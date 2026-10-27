import { LabelBadge } from "@/components/LabelBadge";
import { UserChip } from "@/components/UserChip";
import { ActivityIssueLink } from "../ActivityIssueLink";
import type { LabelsChangedActivityFragment } from "./LabelsChangedActivityFragment.graphql";

export const LabelsChangedActivity: React.FC<{
  event: LabelsChangedActivityFragment;
}> = ({ event }) => (
  <>
    <UserChip user={event.actor} />{" "}
    {event.addedLabels.length > 0 ? (
      <>
        added{" "}
        {event.addedLabels.map((label) => (
          <LabelBadge key={label.id} label={label} />
        ))}{" "}
      </>
    ) : null}
    {event.addedLabels.length > 0 && event.removedLabels.length > 0
      ? "and "
      : null}
    {event.removedLabels.length > 0 ? (
      <>
        removed{" "}
        {event.removedLabels.map((label) => (
          <LabelBadge key={label.id} label={label} />
        ))}{" "}
      </>
    ) : null}
    {event.addedLabels.length > 0 ? "to" : "from"}{" "}
    <ActivityIssueLink issue={event.issue} />
  </>
);
