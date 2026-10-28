import { StateIcon } from "@/components/StateIcon";
import { BoardCard } from "../BoardCard";
import type {
  BoardCard_issue,
  BoardCard_state,
} from "../BoardCard/BoardCardFragment.graphql";
import type { BoardColumnFragment } from "./BoardColumnFragment.graphql";
import classes from "./BoardColumn.module.css";

/** A column of the board, which has issues in one workflow state. */
export const BoardColumn: React.FC<{
  state: BoardColumnFragment;
  issues: readonly BoardCard_issue[];
  states: readonly BoardCard_state[];
}> = ({ state, issues, states }) => (
  <section className={classes.column}>
    <h3 className={classes.header}>
      <StateIcon state={state} showName />
      <span className="muted">{issues.length}</span>
    </h3>
    <div className={classes.cards}>
      {issues.map((issue) => (
        <BoardCard key={issue.id} issue={issue} states={states} />
      ))}
    </div>
  </section>
);
