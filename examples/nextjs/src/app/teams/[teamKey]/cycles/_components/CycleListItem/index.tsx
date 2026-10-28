import Link from "next/link";
import { CycleDates } from "../CycleDates";
import { CycleProgress } from "../CycleProgress";
import type { CycleListItemFragment } from "./CycleListItemFragment.graphql";
import classes from "./CycleListItem.module.css";

export const CycleListItem: React.FC<{
  teamKey: string;
  cycle: CycleListItemFragment;
}> = ({ teamKey, cycle }) => (
  <li className={classes.item}>
    <Link
      href={`/teams/${teamKey}/cycles/${cycle.number}`}
      className={classes.name}
    >
      {cycle.name ?? `Cycle ${cycle.number}`}
    </Link>
    <CycleDates cycle={cycle} />
    <span className={`${classes.count} muted`}>
      {cycle.issues.totalCount} issues
    </span>
    <CycleProgress cycle={cycle} />
  </li>
);
