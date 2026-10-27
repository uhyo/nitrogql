import Link from "next/link";
import { Avatar } from "../Avatar";
import type { UserChipFragment } from "./UserChipFragment.graphql";
import classes from "./UserChip.module.css";

/** Avatar and name of a user, linking to the profile. */
export const UserChip: React.FC<{
  user: UserChipFragment;
  showStatus?: boolean;
}> = ({ user, showStatus = false }) => (
  <Link
    href={`/users/${user.handle}`}
    className={classes.chip}
    data-deactivated={user.deactivated || undefined}
  >
    <Avatar user={user} />
    <span className={classes.name}>{user.name}</span>
    {showStatus && user.status !== null ? (
      <span title={user.status.message}>{user.status.emoji}</span>
    ) : null}
  </Link>
);
