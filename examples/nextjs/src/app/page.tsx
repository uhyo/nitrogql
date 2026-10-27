"use client";

import { useState } from "react";
import { useQuery } from "urql";
import {
  IssueFilters,
  initialIssueFilterState,
  toIssueFilter,
} from "@/components/IssueFilters";
import { IssueList } from "@/components/IssueList";
import MyIssuesQuery from "./myIssues.graphql";

type Tab = "assigned" | "created" | "subscribed";

const pageSize = 50;

export default function MyIssuesPage() {
  const [tab, setTab] = useState<Tab>("assigned");
  const [filter, setFilter] = useState(initialIssueFilterState);
  const [first, setFirst] = useState(pageSize);
  const [{ data, fetching, error }] = useQuery({
    query: MyIssuesQuery,
    variables: {
      assigned: tab === "assigned",
      created: tab === "created",
      subscribed: tab === "subscribed",
      filter: toIssueFilter(filter),
      first,
    },
  });
  const issues =
    data?.viewer.assignedIssues ??
    data?.viewer.createdIssues ??
    data?.viewer.subscribedIssues;

  return (
    <div className="page">
      <header className="pageHeader">
        <h1>My issues</h1>
      </header>
      <IssueFilters value={filter} onChange={setFilter}>
        <select
          className="input"
          value={tab}
          aria-label="Issues to show"
          onChange={(e) => {
            setTab(e.target.value as Tab);
            setFirst(pageSize);
          }}
        >
          <option value="assigned">Assigned to me</option>
          <option value="created">Created by me</option>
          <option value="subscribed">Subscribed</option>
        </select>
      </IssueFilters>
      {error ? <p className="muted">{error.message}</p> : null}
      {issues ? (
        <IssueList
          issues={issues}
          groupByState
          onLoadMore={() => setFirst((first) => first + pageSize)}
        />
      ) : fetching ? (
        <p className="muted">Loading…</p>
      ) : null}
    </div>
  );
}
