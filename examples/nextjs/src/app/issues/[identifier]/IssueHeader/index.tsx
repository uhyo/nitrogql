"use client";

import Link from "next/link";
import { useState } from "react";
import { useMutation } from "urql";
import { formatDateTime } from "@/lib/format";
import { IssueSubscription } from "../IssueSubscription";
import type { IssueHeaderFragment } from "./IssueHeaderFragment.graphql";
import ArchiveIssueMutation from "./archiveIssue.graphql";
import UpdateIssueTitleMutation from "./updateIssueTitle.graphql";
import classes from "./IssueHeader.module.css";

/** Breadcrumbs, title and actions of an issue. */
export const IssueHeader: React.FC<{ issue: IssueHeaderFragment }> = ({
  issue,
}) => {
  const [{ fetching: archiving }, archiveIssue] =
    useMutation(ArchiveIssueMutation);
  return (
    <header className={classes.header}>
      <nav className={classes.breadcrumbs} aria-label="Breadcrumbs">
        <Link href={`/teams/${issue.team.key}/issues`}>
          {issue.team.icon} {issue.team.name}
        </Link>
        {issue.parent !== null ? (
          <>
            <span className="muted">›</span>
            <Link
              href={`/issues/${issue.parent.identifier}`}
              title={issue.parent.title}
            >
              <span className="mono muted">{issue.parent.identifier}</span>{" "}
              {issue.parent.title}
            </Link>
          </>
        ) : null}
        <span className="muted">›</span>
        <span className="mono">{issue.identifier}</span>
        <span className={classes.actions}>
          <IssueSubscription issue={issue} />
          {issue.archivedAt === null ? (
            <button
              type="button"
              className="button"
              disabled={archiving}
              onClick={() => {
                if (confirm(`Archive ${issue.identifier}?`)) {
                  archiveIssue({ id: issue.id });
                }
              }}
            >
              Archive
            </button>
          ) : null}
        </span>
      </nav>
      {issue.archivedAt !== null ? (
        <p className={`${classes.archived} muted`}>
          This issue was archived on {formatDateTime(issue.archivedAt)}.
        </p>
      ) : null}
      <IssueTitle issueId={issue.id} title={issue.title} />
    </header>
  );
};

const IssueTitle: React.FC<{ issueId: string; title: string }> = ({
  issueId,
  title,
}) => {
  const [{ fetching }, updateTitle] = useMutation(UpdateIssueTitleMutation);
  const [draft, setDraft] = useState<string | null>(null);
  if (draft === null) {
    return (
      <h1 className={classes.title}>
        <button
          type="button"
          title="Edit title"
          onClick={() => setDraft(title)}
        >
          {title}
        </button>
      </h1>
    );
  }
  const save = async () => {
    const newTitle = draft.trim();
    if (newTitle !== "" && newTitle !== title) {
      const result = await updateTitle({ id: issueId, title: newTitle });
      if (result.error) {
        return;
      }
    }
    setDraft(null);
  };
  return (
    <form
      className={classes.titleRow}
      onSubmit={(e) => {
        e.preventDefault();
        save();
      }}
    >
      <input
        className={`input ${classes.titleInput}`}
        value={draft}
        aria-label="Title"
        autoFocus
        onChange={(e) => setDraft(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Escape") {
            setDraft(null);
          }
        }}
      />
      <button type="submit" className="button" data-primary disabled={fetching}>
        Save
      </button>
      <button type="button" className="button" onClick={() => setDraft(null)}>
        Cancel
      </button>
    </form>
  );
};
