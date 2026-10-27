import Link from "next/link";
import type { SearchTeamResultFragment } from "./SearchTeamResultFragment.graphql";
import classes from "../SearchResultItem.module.css";

export const SearchTeamResult: React.FC<{
  team: SearchTeamResultFragment;
}> = ({ team }) => (
  <div className={classes.row}>
    <Link href={`/teams/${team.key}/issues`} className={classes.title}>
      <span aria-hidden>{team.icon}</span> {team.name}
    </Link>
    <span className="mono muted" style={{ color: team.color }}>
      {team.key}
    </span>
    <span className={`${classes.description} muted`}>{team.description}</span>
    <span className={`${classes.meta} muted`}>
      {team.members.length} members
    </span>
  </div>
);
