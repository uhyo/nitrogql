import { ProgressBar } from "@/components/ProgressBar";
import type { CycleProgressFragment } from "./CycleProgressFragment.graphql";

/** Progress of a cycle with its completed and total scope. */
export const CycleProgress: React.FC<{ cycle: CycleProgressFragment }> = ({
  cycle,
}) => (
  <span style={{ display: "inline-flex", alignItems: "center", gap: "0.75em" }}>
    <ProgressBar progress={cycle.progress} />
    <span className="muted" title="Completed scope / total scope">
      {cycle.completedScope} / {cycle.scope} pt
    </span>
  </span>
);
