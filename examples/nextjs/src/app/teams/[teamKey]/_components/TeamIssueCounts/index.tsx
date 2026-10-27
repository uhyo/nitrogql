import type { WorkflowStateType } from "@/schema/base.graphql";
import type { TeamIssueCountsFragment } from "./TeamIssueCountsFragment.graphql";
import classes from "./TeamIssueCounts.module.css";

const stateTypeLabels: Record<WorkflowStateType, string> = {
  TRIAGE: "Triage",
  BACKLOG: "Backlog",
  UNSTARTED: "Todo",
  STARTED: "In progress",
  COMPLETED: "Done",
  CANCELED: "Canceled",
};

/** Number of issues of a team per workflow state type. */
export const TeamIssueCounts: React.FC<{ team: TeamIssueCountsFragment }> = ({
  team,
}) => (
  <dl className={classes.counts}>
    {team.issueCounts.map(({ stateType, count }) => (
      <div key={stateType} className={classes.count} data-type={stateType}>
        <dt className="muted">{stateTypeLabels[stateType]}</dt>
        <dd>{count}</dd>
      </div>
    ))}
  </dl>
);
