"use client";

import { useParams } from "next/navigation";
import { useState } from "react";
import { useQuery } from "urql";
import {
  IssueFilters,
  initialIssueFilterState,
  toIssueFilter,
} from "@/components/IssueFilters";
import { IssueList } from "@/components/IssueList";
import { RecentActivity } from "./RecentActivity";
import { UserProfileHeader } from "./UserProfileHeader";
import { UserStatusEditor } from "./UserStatusEditor";
import UserProfileQuery from "./userProfile.graphql";

const pageSize = 20;

export default function UserProfilePage() {
  const { handle } = useParams<{ handle: string }>();
  const [filter, setFilter] = useState(initialIssueFilterState);
  const [first, setFirst] = useState(pageSize);
  const [{ data, fetching, error }] = useQuery({
    query: UserProfileQuery,
    variables: {
      handle: decodeURIComponent(handle),
      filter: toIssueFilter(filter),
      first,
    },
  });

  if (error) {
    return (
      <div className="page">
        <p className="muted">{error.message}</p>
      </div>
    );
  }
  if (!data) {
    return (
      <div className="page">
        {fetching ? <p className="muted">Loading…</p> : null}
      </div>
    );
  }
  const { user, viewer } = data;
  if (user === null) {
    return (
      <div className="page">
        <h1>User not found</h1>
        <p className="muted">There is no user with handle @{handle}.</p>
      </div>
    );
  }
  const isViewer = user.id === viewer.id;

  return (
    <div className="page">
      <UserProfileHeader user={user}>
        {isViewer ? <UserStatusEditor user={user} /> : null}
      </UserProfileHeader>
      <section className="section">
        <h2>Assigned issues</h2>
        <IssueFilters
          value={filter}
          onChange={(filter) => {
            setFilter(filter);
            setFirst(pageSize);
          }}
        />
        <IssueList
          issues={user.assignedIssues}
          emptyMessage={`No issues assigned to ${user.name}`}
          onLoadMore={() => setFirst((first) => first + pageSize)}
        />
      </section>
      <section className="section">
        <h2>Recent activity</h2>
        <RecentActivity events={user.recentActivity} />
      </section>
    </div>
  );
}
