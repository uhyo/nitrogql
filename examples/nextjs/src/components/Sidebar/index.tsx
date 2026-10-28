"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useQuery } from "urql";
import { UserChip } from "../UserChip";
import SidebarQuery from "./sidebar.graphql";
import classes from "./Sidebar.module.css";

export const Sidebar: React.FC = () => {
  const [{ data }] = useQuery({ query: SidebarQuery });
  const pathname = usePathname();
  const navLink = (href: string, label: React.ReactNode, exact = false) => (
    <Link
      href={href}
      className={classes.link}
      aria-current={
        (exact ? pathname === href : pathname.startsWith(href))
          ? "page"
          : undefined
      }
    >
      {label}
    </Link>
  );
  return (
    <nav className={classes.sidebar}>
      <div className={classes.viewer}>
        {data ? <UserChip user={data.viewer} showStatus /> : null}
      </div>
      <form action="/search" className={classes.search}>
        <input
          type="search"
          name="q"
          placeholder="Search…"
          className="input"
          aria-label="Search"
        />
      </form>
      <div className={classes.group}>
        {navLink(
          "/inbox",
          <>
            Inbox
            {data && data.notifications.unreadCount > 0 ? (
              <span className={classes.count}>
                {data.notifications.unreadCount}
              </span>
            ) : null}
          </>,
        )}
        {navLink("/", "My issues", true)}
        {navLink("/projects", "Projects")}
      </div>
      <div className={classes.group}>
        <h2 className={classes.groupTitle}>Your teams</h2>
        {data?.viewer.teams.map((team) => (
          <div key={team.id} className={classes.team}>
            <span className={classes.teamName}>
              {team.icon} {team.name}
            </span>
            {navLink(`/teams/${team.key}/issues`, "Issues")}
            {navLink(`/teams/${team.key}/board`, "Board")}
            {team.activeCycle !== null
              ? navLink(
                  `/teams/${team.key}/cycles/${team.activeCycle.number}`,
                  `Cycle ${team.activeCycle.number}`,
                )
              : null}
          </div>
        ))}
      </div>
    </nav>
  );
};
