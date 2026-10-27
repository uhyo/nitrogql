import type { AvatarFragment } from "./AvatarFragment.graphql";
import classes from "./Avatar.module.css";

export const Avatar: React.FC<{
  user: AvatarFragment;
  size?: number;
}> = ({ user, size = 20 }) => {
  const style = { width: size, height: size, fontSize: size * 0.45 };
  if (user.avatarUrl === null) {
    return (
      <span className={classes.avatar} style={style} title={user.name}>
        {initials(user.name)}
      </span>
    );
  }
  return (
    // eslint-disable-next-line @next/next/no-img-element
    <img
      className={classes.avatar}
      style={style}
      src={user.avatarUrl}
      alt={user.name}
      title={user.name}
    />
  );
};

function initials(name: string): string {
  return name
    .split(/\s+/)
    .map((word) => word[0])
    .slice(0, 2)
    .join("")
    .toUpperCase();
}
