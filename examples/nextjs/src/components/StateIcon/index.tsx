import type { StateIconFragment } from "./StateIconFragment.graphql";
import classes from "./StateIcon.module.css";

/** Icon of a workflow state. With `showName`, the name of the state is shown too. */
export const StateIcon: React.FC<{
  state: StateIconFragment;
  showName?: boolean;
}> = ({ state, showName = false }) => {
  const fill = {
    TRIAGE: 0,
    BACKLOG: 0,
    UNSTARTED: 0,
    STARTED: 0.5,
    COMPLETED: 1,
    CANCELED: 1,
  }[state.type];
  return (
    <span className={classes.state} title={state.name}>
      <svg width="14" height="14" viewBox="0 0 14 14" aria-hidden>
        <circle
          cx="7"
          cy="7"
          r="6"
          fill="none"
          stroke={state.color}
          strokeWidth="1.5"
          strokeDasharray={state.type === "BACKLOG" ? "2 2" : undefined}
        />
        {fill > 0 ? (
          <path
            d={
              fill === 1
                ? "M7 3a4 4 0 1 1 0 8a4 4 0 1 1 0-8"
                : "M7 3a4 4 0 0 1 0 8z"
            }
            fill={state.color}
          />
        ) : null}
        {state.type === "CANCELED" ? (
          <path d="M5 5l4 4M9 5l-4 4" stroke="white" strokeWidth="1.2" />
        ) : null}
      </svg>
      {showName ? <span>{state.name}</span> : null}
    </span>
  );
};
