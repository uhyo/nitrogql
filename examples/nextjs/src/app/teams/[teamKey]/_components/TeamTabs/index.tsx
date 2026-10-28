"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import classes from "./TeamTabs.module.css";

const tabs = [
  { path: "issues", label: "Issues" },
  { path: "board", label: "Board" },
  { path: "cycles", label: "Cycles" },
  { path: "members", label: "Members" },
] as const;

export const TeamTabs: React.FC<{ teamKey: string }> = ({ teamKey }) => {
  const pathname = usePathname();
  return (
    <nav className={classes.tabs}>
      {tabs.map(({ path, label }) => {
        const href = `/teams/${teamKey}/${path}`;
        return (
          <Link
            key={path}
            href={href}
            className={classes.tab}
            aria-current={
              pathname === href || pathname.startsWith(`${href}/`)
                ? "page"
                : undefined
            }
          >
            {label}
          </Link>
        );
      })}
    </nav>
  );
};
