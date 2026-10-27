"use client";

import { useParams } from "next/navigation";
import { useQuery } from "urql";
import { CycleListItem } from "./_components/CycleListItem";
import TeamCyclesQuery from "./teamCycles.graphql";
import classes from "./cycles.module.css";

export default function TeamCyclesPage() {
  const { teamKey } = useParams<{ teamKey: string }>();
  const [{ data, fetching, error }] = useQuery({
    query: TeamCyclesQuery,
    variables: { teamKey },
  });

  if (error) {
    return <p className="muted">{error.message}</p>;
  }
  if (!data) {
    return fetching ? <p className="muted">Loading…</p> : null;
  }
  if (data.team === null) {
    return <p className="muted">Team not found.</p>;
  }
  const { team } = data;
  return (
    <>
      <div className="pageHeader">
        <h2>Cycles</h2>
      </div>
      {team.cycles.length === 0 ? (
        <p className={`${classes.empty} muted`}>
          {team.name} does not use cycles.
        </p>
      ) : (
        <ul className={classes.list}>
          {team.cycles.map((cycle) => (
            <CycleListItem key={cycle.id} teamKey={teamKey} cycle={cycle} />
          ))}
        </ul>
      )}
    </>
  );
}
