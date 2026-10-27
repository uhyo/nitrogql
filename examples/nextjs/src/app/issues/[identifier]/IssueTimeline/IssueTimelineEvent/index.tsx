import { RelativeTime } from "@/components/RelativeTime";
import { UserChip } from "@/components/UserChip";
import type { IssueTimelineEventFragment } from "./IssueTimelineEventFragment.graphql";
import classes from "./IssueTimelineEvent.module.css";

/** Common layout of an event in the timeline: "<icon> <actor> <children> · <time>". */
export const IssueTimelineEvent: React.FC<{
  event: IssueTimelineEventFragment;
  icon: React.ReactNode;
  children: React.ReactNode;
}> = ({ event, icon, children }) => (
  <div className={classes.event}>
    <span className={classes.icon} aria-hidden>
      {icon}
    </span>
    <UserChip user={event.actor} />
    {children}
    <span className="muted">·</span>
    <RelativeTime date={event.createdAt} className="muted" />
  </div>
);
