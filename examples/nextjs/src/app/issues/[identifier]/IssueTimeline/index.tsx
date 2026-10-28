import { IssueCommentComposer } from "./IssueCommentComposer";
import { IssueTimelineAssigneeChanged } from "./IssueTimelineAssigneeChanged";
import { IssueTimelineComment } from "./IssueTimelineComment";
import { IssueTimelineCreated } from "./IssueTimelineCreated";
import { IssueTimelineLabelsChanged } from "./IssueTimelineLabelsChanged";
import { IssueTimelinePriorityChanged } from "./IssueTimelinePriorityChanged";
import { IssueTimelineProjectChanged } from "./IssueTimelineProjectChanged";
import { IssueTimelineStateChanged } from "./IssueTimelineStateChanged";
import type { IssueTimelineFragment } from "./IssueTimelineFragment.graphql";
import classes from "./IssueTimeline.module.css";

type TimelineItem = IssueTimelineFragment["timeline"][number];

/** Comments and events of an issue, oldest first, followed by a comment form. */
export const IssueTimeline: React.FC<{ issue: IssueTimelineFragment }> = ({
  issue,
}) => (
  <section className="section">
    <h2>Activity</h2>
    <ol className={classes.timeline}>
      {issue.timeline.map((item) => (
        <li key={item.id}>
          <TimelineItem item={item} issueId={issue.id} />
        </li>
      ))}
    </ol>
    <IssueCommentComposer issueId={issue.id} />
  </section>
);

const TimelineItem: React.FC<{ item: TimelineItem; issueId: string }> = ({
  item,
  issueId,
}) => {
  switch (item.__typename) {
    case "Comment":
      return <IssueTimelineComment comment={item} issueId={issueId} />;
    case "IssueCreatedEvent":
      return <IssueTimelineCreated event={item} />;
    case "StateChangedEvent":
      return <IssueTimelineStateChanged event={item} />;
    case "AssigneeChangedEvent":
      return <IssueTimelineAssigneeChanged event={item} />;
    case "PriorityChangedEvent":
      return <IssueTimelinePriorityChanged event={item} />;
    case "LabelsChangedEvent":
      return <IssueTimelineLabelsChanged event={item} />;
    case "ProjectChangedEvent":
      return <IssueTimelineProjectChanged event={item} />;
  }
};
