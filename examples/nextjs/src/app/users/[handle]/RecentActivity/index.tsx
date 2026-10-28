import { RelativeTime } from "@/components/RelativeTime";
import { AssigneeChangedActivity } from "./AssigneeChangedActivity";
import { CommentedActivity } from "./CommentedActivity";
import { IssueCreatedActivity } from "./IssueCreatedActivity";
import { LabelsChangedActivity } from "./LabelsChangedActivity";
import { PriorityChangedActivity } from "./PriorityChangedActivity";
import { ProjectChangedActivity } from "./ProjectChangedActivity";
import { StateChangedActivity } from "./StateChangedActivity";
import type { RecentActivityFragment } from "./RecentActivityFragment.graphql";
import classes from "./RecentActivity.module.css";

const icons: Record<RecentActivityFragment["__typename"], string> = {
  IssueCreatedEvent: "＋",
  StateChangedEvent: "◐",
  AssigneeChangedEvent: "👤",
  PriorityChangedEvent: "⇅",
  LabelsChangedEvent: "🏷",
  ProjectChangedEvent: "▦",
  CommentedEvent: "💬",
};

/** Recent activity of a user, newest first. */
export const RecentActivity: React.FC<{
  events: readonly RecentActivityFragment[];
}> = ({ events }) => {
  if (events.length === 0) {
    return <p className={`${classes.empty} muted`}>No recent activity</p>;
  }
  return (
    <ol className={classes.list}>
      {events.map((event) => (
        <li key={event.id} className={classes.item}>
          <span className={classes.icon} aria-hidden>
            {icons[event.__typename]}
          </span>
          <div className={classes.body}>
            <ActivityBody event={event} />
          </div>
          <RelativeTime date={event.createdAt} className="muted" />
        </li>
      ))}
    </ol>
  );
};

const ActivityBody: React.FC<{ event: RecentActivityFragment }> = ({
  event,
}) => {
  switch (event.__typename) {
    case "IssueCreatedEvent":
      return <IssueCreatedActivity event={event} />;
    case "StateChangedEvent":
      return <StateChangedActivity event={event} />;
    case "AssigneeChangedEvent":
      return <AssigneeChangedActivity event={event} />;
    case "PriorityChangedEvent":
      return <PriorityChangedActivity event={event} />;
    case "LabelsChangedEvent":
      return <LabelsChangedActivity event={event} />;
    case "ProjectChangedEvent":
      return <ProjectChangedActivity event={event} />;
    case "CommentedEvent":
      return <CommentedActivity event={event} />;
  }
};
