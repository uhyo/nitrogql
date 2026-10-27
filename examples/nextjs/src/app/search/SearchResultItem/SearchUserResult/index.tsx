import { UserChip } from "@/components/UserChip";
import type { SearchUserResultFragment } from "./SearchUserResultFragment.graphql";
import classes from "../SearchResultItem.module.css";

export const SearchUserResult: React.FC<{
  user: SearchUserResultFragment;
}> = ({ user }) => (
  <div className={classes.row}>
    <UserChip user={user} showStatus />
    <span className="mono muted">@{user.handle}</span>
    <span className={`${classes.description} muted`}>{user.email}</span>
    <span className={classes.meta}>
      {user.teams.map((team) => (
        <span key={team.id} title={team.key}>
          {team.icon}
        </span>
      ))}
      {user.deactivated ? <span className="muted">Deactivated</span> : null}
    </span>
  </div>
);
