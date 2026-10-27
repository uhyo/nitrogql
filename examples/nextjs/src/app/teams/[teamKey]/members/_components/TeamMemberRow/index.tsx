import Link from "next/link";
import { RelativeTime } from "@/components/RelativeTime";
import { UserChip } from "@/components/UserChip";
import type { TeamRole } from "@/schema/base.graphql";
import type { TeamMemberRowFragment } from "./TeamMemberRowFragment.graphql";
import classes from "./TeamMemberRow.module.css";

const roleLabels: Record<TeamRole, string> = {
  OWNER: "Owner",
  ADMIN: "Admin",
  MEMBER: "Member",
  GUEST: "Guest",
};

export const TeamMemberRow: React.FC<{
  teamKey: string;
  membership: TeamMemberRowFragment;
}> = ({ teamKey, membership: { user, role, joinedAt } }) => (
  <tr>
    <td>
      <UserChip user={user} showStatus />
    </td>
    <td>
      <span className={classes.role} data-role={role}>
        {roleLabels[role]}
      </span>
    </td>
    <td className="muted">{user.email}</td>
    <td className="muted">{user.timezone}</td>
    <td>
      Joined <RelativeTime date={joinedAt} />
    </td>
    <td>
      <Link
        href={`/teams/${teamKey}/issues?assignee=${user.id}`}
        className="muted"
      >
        Issues →
      </Link>
    </td>
  </tr>
);
