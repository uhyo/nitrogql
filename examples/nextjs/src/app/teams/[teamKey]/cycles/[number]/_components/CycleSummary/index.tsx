import { formatDateTime } from "@/lib/format";
import { CycleDates } from "../../../_components/CycleDates";
import { CycleProgress } from "../../../_components/CycleProgress";
import type { CycleSummaryFragment } from "./CycleSummaryFragment.graphql";
import classes from "./CycleSummary.module.css";

export const CycleSummary: React.FC<{ cycle: CycleSummaryFragment }> = ({
  cycle,
}) => (
  <section className={classes.summary}>
    <h2>
      {cycle.name ?? `Cycle ${cycle.number}`}
      {cycle.name !== null ? (
        <span className="muted"> #{cycle.number}</span>
      ) : null}
    </h2>
    <dl className={classes.properties}>
      <dt className="muted">Dates</dt>
      <dd>
        <CycleDates cycle={cycle} />
      </dd>
      {cycle.completedAt !== null ? (
        <>
          <dt className="muted">Completed</dt>
          <dd>{formatDateTime(cycle.completedAt)}</dd>
        </>
      ) : null}
      <dt className="muted">Progress</dt>
      <dd>
        <CycleProgress cycle={cycle} />
      </dd>
      <dt className="muted">Remaining</dt>
      <dd>{cycle.scope - cycle.completedScope} pt</dd>
    </dl>
  </section>
);
