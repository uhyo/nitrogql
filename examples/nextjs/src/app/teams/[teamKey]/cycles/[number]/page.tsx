"use client";

import Link from "next/link";
import { useParams } from "next/navigation";
import { useState } from "react";
import { useQuery } from "urql";
import {
  IssueFilters,
  type IssueFilterState,
  initialIssueFilterState,
  toIssueFilter,
} from "@/components/IssueFilters";
import { IssueList } from "@/components/IssueList";
import { CycleAssigneeBreakdown } from "./_components/CycleAssigneeBreakdown";
import { CycleSummary } from "./_components/CycleSummary";
import CycleDetailQuery from "./cycleDetail.graphql";

const pageSize = 50;

export default function CycleDetailPage() {
  const params = useParams<{ teamKey: string; number: string }>();
  const number = Number(params.number);
  const isValidNumber = Number.isInteger(number) && number > 0;
  const [filter, setFilter] = useState<IssueFilterState>({
    ...initialIssueFilterState,
    tab: "all",
  });
  const [first, setFirst] = useState(pageSize);
  const [{ data, fetching, error }] = useQuery({
    query: CycleDetailQuery,
    variables: {
      teamKey: params.teamKey,
      number,
      filter: toIssueFilter(filter),
      first,
    },
    pause: !isValidNumber,
  });

  if (!isValidNumber || data?.cycle === null) {
    return (
      <p className="muted">
        Cycle {params.number} not found.{" "}
        <Link href={`/teams/${params.teamKey}/cycles`}>See all cycles</Link>
      </p>
    );
  }
  if (error) {
    return <p className="muted">{error.message}</p>;
  }
  if (!data) {
    return fetching ? <p className="muted">Loading…</p> : null;
  }
  const { cycle } = data;
  return (
    <>
      <CycleSummary cycle={cycle} />
      <section className="section">
        <h2>Issues</h2>
        <IssueFilters
          value={filter}
          onChange={(filter) => {
            setFilter(filter);
            setFirst(pageSize);
          }}
        />
        <IssueList
          issues={cycle.issues}
          groupByState
          emptyMessage="No issues match the filters"
          onLoadMore={() => setFirst((first) => first + pageSize)}
        />
      </section>
      <section className="section">
        <h2>By assignee</h2>
        <CycleAssigneeBreakdown issues={cycle.allIssues} />
      </section>
    </>
  );
}
