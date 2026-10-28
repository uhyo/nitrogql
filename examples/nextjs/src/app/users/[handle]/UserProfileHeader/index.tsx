"use client";

import Link from "next/link";
import { useEffect, useState } from "react";
import { Avatar } from "@/components/Avatar";
import { formatDateTime } from "@/lib/format";
import type { UserProfileHeaderFragment } from "./UserProfileHeaderFragment.graphql";
import classes from "./UserProfileHeader.module.css";

export const UserProfileHeader: React.FC<{
  user: UserProfileHeaderFragment;
  /** Shown next to the status, such as a status editor. */
  children?: React.ReactNode;
}> = ({ user, children }) => (
  <header className={classes.header}>
    <Avatar user={user} size={72} />
    <div className={classes.main}>
      <h1 className={classes.name}>
        {user.name}
        {user.deactivated ? (
          <span className={classes.deactivated}>Deactivated</span>
        ) : null}
      </h1>
      <p className="muted">
        @{user.handle} · <LocalTime timezone={user.timezone} />
      </p>
      {user.bio !== null ? <p className={classes.bio}>{user.bio}</p> : null}
      <div className={classes.status}>
        {user.status !== null ? (
          <span>
            {user.status.emoji} {user.status.message}
            {user.status.expiresAt !== null ? (
              <span className="muted">
                {" "}
                · until {formatDateTime(user.status.expiresAt)}
              </span>
            ) : null}
          </span>
        ) : null}
        {children}
      </div>
      {user.teams.length > 0 ? (
        <ul className={classes.teams}>
          {user.teams.map((team) => (
            <li key={team.id}>
              <Link href={`/teams/${team.key}/issues`} className="button">
                {team.icon} {team.name}
              </Link>
            </li>
          ))}
        </ul>
      ) : null}
    </div>
  </header>
);

/** Current time in the given time zone, updated every minute. */
const LocalTime: React.FC<{ timezone: string }> = ({ timezone }) => {
  const [now, setNow] = useState(() => new Date());
  useEffect(() => {
    const timer = setInterval(() => setNow(new Date()), 60 * 1000);
    return () => clearInterval(timer);
  }, []);
  const time = new Intl.DateTimeFormat("en", {
    timeZone: timezone,
    hour: "numeric",
    minute: "2-digit",
  }).format(now);
  return (
    <span title={timezone}>
      {time} local time ({timezone.replace(/_/g, " ")})
    </span>
  );
};
