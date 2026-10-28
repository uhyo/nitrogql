"use client";

import { useParams } from "next/navigation";
import { useState } from "react";
import { useQuery } from "urql";
import type { WorkflowStateType } from "@/schema/base.graphql";
import { BoardColumn } from "./_components/BoardColumn";
import TeamBoardQuery from "./teamBoard.graphql";
import classes from "./board.module.css";

/** Canceled issues are not shown on the board. Archived issues are excluded by default. */
const boardStateTypes: WorkflowStateType[] = [
  "TRIAGE",
  "BACKLOG",
  "UNSTARTED",
  "STARTED",
  "COMPLETED",
];

export default function TeamBoardPage() {
  const { teamKey } = useParams<{ teamKey: string }>();
  const [search, setSearch] = useState("");
  const [{ data, fetching, error }] = useQuery({
    query: TeamBoardQuery,
    variables: {
      teamKey,
      filter: { stateTypes: boardStateTypes, search: search.trim() || null },
    },
  });

  if (data?.team === null) {
    return <p className="muted">Team not found.</p>;
  }
  const team = data?.team;
  const columns = team?.workflowStates.filter((state) =>
    boardStateTypes.includes(state.type),
  );
  return (
    <>
      <div className="pageHeader">
        <h2>Board</h2>
        {team ? (
          <span className="muted">{team.issues.totalCount} issues</span>
        ) : null}
        <input
          type="search"
          className={`input ${classes.search}`}
          placeholder="Filter by text…"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
        />
      </div>
      {error ? <p className="muted">{error.message}</p> : null}
      {team && columns ? (
        <div className={classes.board}>
          {columns.map((state) => (
            <BoardColumn
              key={state.id}
              state={state}
              issues={team.issues.nodes.filter(
                (issue) => issue.state.id === state.id,
              )}
              states={team.workflowStates}
            />
          ))}
        </div>
      ) : fetching ? (
        <p className="muted">Loading…</p>
      ) : null}
    </>
  );
}
