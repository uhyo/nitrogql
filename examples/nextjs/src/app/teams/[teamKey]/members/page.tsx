"use client";

import { useParams } from "next/navigation";
import { useQuery } from "urql";
import { TeamLabelRow } from "./_components/TeamLabelRow";
import { TeamMemberRow } from "./_components/TeamMemberRow";
import TeamMembersQuery from "./teamMembers.graphql";
import classes from "./members.module.css";

export default function TeamMembersPage() {
  const { teamKey } = useParams<{ teamKey: string }>();
  const [{ data, fetching, error }] = useQuery({
    query: TeamMembersQuery,
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
      <section className="section">
        <h2>Members ({team.members.length})</h2>
        <table className={classes.table}>
          <tbody>
            {team.members.map((membership) => (
              <TeamMemberRow
                key={membership.user.id}
                teamKey={teamKey}
                membership={membership}
              />
            ))}
          </tbody>
        </table>
      </section>
      <section className="section">
        <h2>Labels ({team.labels.length})</h2>
        <table className={classes.table}>
          <tbody>
            {team.labels.map((label) => (
              <TeamLabelRow key={label.id} teamKey={teamKey} label={label} />
            ))}
          </tbody>
        </table>
      </section>
    </>
  );
}
