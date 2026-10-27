"use client";

import { useParams, useSearchParams } from "next/navigation";
import { useState } from "react";
import { useQuery } from "urql";
import { IssueList } from "@/components/IssueList";
import { NewIssueForm } from "./_components/NewIssueForm";
import {
  TeamIssueFilters,
  TeamIssueSortSelect,
  type TeamIssueFilterState,
  initialTeamIssueFilterState,
  toIssueSort,
  toTeamIssueFilter,
} from "./_components/TeamIssueFilters";
import TeamIssuesQuery from "./teamIssues.graphql";
import classes from "./issues.module.css";

const pageSize = 50;

export default function TeamIssuesPage() {
  const { teamKey } = useParams<{ teamKey: string }>();
  const searchParams = useSearchParams();
  // Links from other pages can preset the label and assignee filters
  const [filter, setFilter] = useState(() => ({
    ...initialTeamIssueFilterState,
    labelId: searchParams.get("label") ?? "",
    assignee: searchParams.get("assignee") ?? "",
  }));
  const [first, setFirst] = useState(pageSize);
  const [showForm, setShowForm] = useState(false);
  const [{ data, fetching, error }] = useQuery({
    query: TeamIssuesQuery,
    variables: {
      teamKey,
      filter: toTeamIssueFilter(filter),
      sort: toIssueSort(filter),
      first,
    },
  });
  const team = data?.team ?? undefined;
  const changeFilter = (filter: TeamIssueFilterState) => {
    setFilter(filter);
    setFirst(pageSize);
  };

  if (data?.team === null) {
    return <p className="muted">Team not found.</p>;
  }
  return (
    <>
      <div className="pageHeader">
        <h2>Issues</h2>
        <span className={classes.spacer} />
        <TeamIssueSortSelect value={filter} onChange={changeFilter} />
        {team && !showForm ? (
          <button
            type="button"
            className="button"
            data-primary
            onClick={() => setShowForm(true)}
          >
            New issue
          </button>
        ) : null}
      </div>
      {team && showForm ? (
        <NewIssueForm team={team} onClose={() => setShowForm(false)} />
      ) : null}
      <TeamIssueFilters team={team} value={filter} onChange={changeFilter} />
      {error ? <p className="muted">{error.message}</p> : null}
      {team ? (
        <IssueList
          issues={team.issues}
          groupByState
          emptyMessage="No issues match the filters"
          onLoadMore={() => setFirst((first) => first + pageSize)}
        />
      ) : fetching ? (
        <p className="muted">Loading…</p>
      ) : null}
    </>
  );
}
