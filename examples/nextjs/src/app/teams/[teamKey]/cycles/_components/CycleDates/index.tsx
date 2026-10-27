import { formatDate, formatRelative } from "@/lib/format";
import type { CycleDatesFragment } from "./CycleDatesFragment.graphql";
import classes from "./CycleDates.module.css";

export type CycleStatus = "upcoming" | "active" | "completed";

export function cycleStatus(
  cycle: CycleDatesFragment,
  now = new Date().toISOString(),
): CycleStatus {
  if (cycle.completedAt !== null || cycle.endsAt <= now) {
    return "completed";
  }
  return cycle.startsAt <= now ? "active" : "upcoming";
}

const statusLabels: Record<CycleStatus, string> = {
  upcoming: "Upcoming",
  active: "Active",
  completed: "Completed",
};

/** Date range and status of a cycle. */
export const CycleDates: React.FC<{ cycle: CycleDatesFragment }> = ({
  cycle,
}) => {
  const status = cycleStatus(cycle);
  return (
    <span className={classes.dates}>
      <span className={classes.status} data-status={status}>
        {statusLabels[status]}
      </span>
      <span>
        {formatDate(cycle.startsAt)} – {formatDate(cycle.endsAt)}
      </span>
      {status === "active" ? (
        <span className="muted">ends {formatRelative(cycle.endsAt)}</span>
      ) : status === "upcoming" ? (
        <span className="muted">starts {formatRelative(cycle.startsAt)}</span>
      ) : null}
    </span>
  );
};
