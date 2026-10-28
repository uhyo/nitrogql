"use client";

import Link from "next/link";
import { useMutation } from "urql";
import { Avatar } from "@/components/Avatar";
import { LabelBadge } from "@/components/LabelBadge";
import { PriorityIcon } from "@/components/PriorityIcon";
import { ProjectChip } from "@/components/ProjectChip";
import { formatDate } from "@/lib/format";
import type {
  BoardCard_issue,
  BoardCard_state,
} from "./BoardCardFragment.graphql";
import MoveIssueMutation from "./moveIssue.graphql";
import classes from "./BoardCard.module.css";

/** A card of an issue on the board. The state select moves the card. */
export const BoardCard: React.FC<{
  issue: BoardCard_issue;
  states: readonly BoardCard_state[];
}> = ({ issue, states }) => {
  const [{ fetching, error }, moveIssue] = useMutation(MoveIssueMutation);
  return (
    <article className={classes.card} aria-busy={fetching}>
      <header className={classes.header}>
        <PriorityIcon priority={issue.priority} />
        <span className="mono muted">{issue.identifier}</span>
        {issue.assignee !== null ? (
          <span className={classes.assignee}>
            <Avatar user={issue.assignee} />
          </span>
        ) : null}
      </header>
      <Link href={`/issues/${issue.identifier}`} className={classes.title}>
        {issue.title}
      </Link>
      {issue.labels.length > 0 || issue.project !== null ? (
        <div className={classes.meta}>
          {issue.project !== null ? (
            <ProjectChip project={issue.project} />
          ) : null}
          {issue.labels.map((label) => (
            <LabelBadge key={label.id} label={label} />
          ))}
        </div>
      ) : null}
      <footer className={classes.footer}>
        <select
          className="input"
          aria-label={`State of ${issue.identifier}`}
          value={issue.state.id}
          disabled={fetching}
          onChange={(e) => moveIssue({ id: issue.id, stateId: e.target.value })}
        >
          {states.map((state) => (
            <option key={state.id} value={state.id}>
              {state.name}
            </option>
          ))}
        </select>
        {issue.dueDate !== null ? (
          <span className="muted" title="Due date">
            📅 {formatDate(issue.dueDate)}
          </span>
        ) : null}
        {issue.estimate !== null ? (
          <span className="muted" title="Estimate">
            {issue.estimate}pt
          </span>
        ) : null}
      </footer>
      {error ? <p className={classes.error}>{error.message}</p> : null}
    </article>
  );
};
