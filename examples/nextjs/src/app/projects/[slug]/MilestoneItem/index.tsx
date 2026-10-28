import { ProgressBar } from "@/components/ProgressBar";
import { formatDate } from "@/lib/format";
import type { MilestoneItemFragment } from "./MilestoneItemFragment.graphql";
import classes from "./MilestoneItem.module.css";

/** One milestone in the list of milestones of a project. */
export const MilestoneItem: React.FC<{
  milestone: MilestoneItemFragment;
  color: string;
}> = ({ milestone, color }) => {
  const completed = milestone.issues.filter(
    (issue) =>
      issue.state.type === "COMPLETED" || issue.state.type === "CANCELED",
  ).length;
  return (
    <li className={classes.milestone}>
      <span aria-hidden>◆</span>
      <div className={classes.name}>
        <strong>{milestone.name}</strong>
        {milestone.description !== null ? (
          <p className="muted">{milestone.description}</p>
        ) : null}
      </div>
      <span className={`${classes.meta} muted`}>
        <span title="Completed issues">
          {completed} / {milestone.issues.length} issues
        </span>
        {milestone.targetDate !== null ? (
          <span title="Target date">🎯 {formatDate(milestone.targetDate)}</span>
        ) : null}
        <ProgressBar progress={milestone.progress} color={color} />
      </span>
    </li>
  );
};
