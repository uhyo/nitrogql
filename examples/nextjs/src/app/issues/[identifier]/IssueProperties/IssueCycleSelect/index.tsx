import Link from "next/link";
import { formatDate } from "@/lib/format";
import type { UpdateIssueInput } from "@/schema/base.graphql";
import type { IssueCycleSelectFragment } from "./IssueCycleSelectFragment.graphql";

const noCycle = "";

/** Selects a cycle of the issue's team. */
export const IssueCycleSelect: React.FC<{
  issue: IssueCycleSelectFragment;
  disabled?: boolean;
  onChange: (input: UpdateIssueInput) => void;
}> = ({ issue, disabled, onChange }) => {
  const { cycle, team } = issue;
  // `team.cycles` does not include completed cycles
  const isCurrentListed =
    cycle === null || team.cycles.some((c) => c.id === cycle.id);
  return (
    <>
      <select
        className="input"
        aria-label="Cycle"
        value={cycle?.id ?? noCycle}
        disabled={disabled}
        onChange={(e) =>
          onChange({
            cycleId: e.target.value === noCycle ? null : e.target.value,
          })
        }
      >
        <option value={noCycle}>No cycle</option>
        {!isCurrentListed ? (
          <option value={cycle.id}>{cycleName(cycle)}</option>
        ) : null}
        {team.cycles.map((cycle) => (
          <option key={cycle.id} value={cycle.id}>
            {`${cycleName(cycle)} (${formatDate(cycle.startsAt)} – ${formatDate(cycle.endsAt)})`}
          </option>
        ))}
      </select>
      {cycle !== null ? (
        <Link
          href={`/teams/${team.key}/cycles/${cycle.number}`}
          title="Open cycle"
        >
          ↗
        </Link>
      ) : null}
    </>
  );
};

function cycleName(cycle: { number: number; name: string | null }): string {
  return cycle.name ?? `Cycle ${cycle.number}`;
}
