import { Avatar } from "@/components/Avatar";
import { TeamIssueCounts } from "../TeamIssueCounts";
import type { TeamHeaderFragment } from "./TeamHeaderFragment.graphql";
import classes from "./TeamHeader.module.css";

/** Number of member avatars shown before "+N". */
const maxAvatars = 6;

export const TeamHeader: React.FC<{ team: TeamHeaderFragment }> = ({
  team,
}) => {
  const members = team.members.filter(({ user }) => !user.deactivated);
  return (
    <header className={classes.header}>
      <div className={classes.title}>
        <span
          className={classes.icon}
          style={{ background: team.color }}
          aria-hidden
        >
          {team.icon}
        </span>
        <div>
          <h1>
            {team.name} <span className="mono muted">{team.key}</span>
          </h1>
          {team.description !== null ? (
            <p className="muted">{team.description}</p>
          ) : null}
        </div>
        <div
          className={classes.members}
          title={`${members.length} members`}
          aria-label={`${members.length} members`}
        >
          {members.slice(0, maxAvatars).map(({ user }) => (
            <Avatar key={user.id} user={user} size={24} />
          ))}
          {members.length > maxAvatars ? (
            <span className="muted">+{members.length - maxAvatars}</span>
          ) : null}
        </div>
      </div>
      <TeamIssueCounts team={team} />
    </header>
  );
};
