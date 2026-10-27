"use client";

import { useParams } from "next/navigation";
import { useQuery } from "urql";
import { TeamHeader } from "./_components/TeamHeader";
import { TeamTabs } from "./_components/TeamTabs";
import TeamLayoutQuery from "./teamLayout.graphql";

export default function TeamLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const { teamKey } = useParams<{ teamKey: string }>();
  const [{ data, fetching, error }] = useQuery({
    query: TeamLayoutQuery,
    variables: { teamKey },
  });

  if (data?.team === null) {
    return (
      <div className="page">
        <p className="muted">Team {teamKey} not found.</p>
      </div>
    );
  }
  return (
    <div className="page">
      {error ? <p className="muted">{error.message}</p> : null}
      {data ? (
        <TeamHeader team={data.team} />
      ) : fetching ? (
        <p className="muted">Loading…</p>
      ) : null}
      <TeamTabs teamKey={teamKey} />
      {children}
    </div>
  );
}
