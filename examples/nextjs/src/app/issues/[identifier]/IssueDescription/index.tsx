"use client";

import { useState } from "react";
import { useMutation } from "urql";
import { Markdown } from "@/components/Markdown";
import type { IssueDescriptionFragment } from "./IssueDescriptionFragment.graphql";
import UpdateIssueDescriptionMutation from "./updateIssueDescription.graphql";
import classes from "./IssueDescription.module.css";

/** Description of an issue, editable in place. */
export const IssueDescription: React.FC<{
  issue: IssueDescriptionFragment;
}> = ({ issue }) => {
  const [{ fetching, error }, updateDescription] = useMutation(
    UpdateIssueDescriptionMutation,
  );
  const [draft, setDraft] = useState<string | null>(null);

  if (draft !== null) {
    return (
      <form
        className={classes.editor}
        onSubmit={async (e) => {
          e.preventDefault();
          const result = await updateDescription({
            id: issue.id,
            description: draft.trim() || null,
          });
          if (!result.error) {
            setDraft(null);
          }
        }}
      >
        <textarea
          className="input"
          value={draft}
          aria-label="Description"
          placeholder="Add a description… (Markdown is supported)"
          autoFocus
          onChange={(e) => setDraft(e.target.value)}
        />
        {error ? <p className="muted">{error.message}</p> : null}
        <div className={classes.buttons}>
          <button
            type="button"
            className="button"
            onClick={() => setDraft(null)}
          >
            Cancel
          </button>
          <button
            type="submit"
            className="button"
            data-primary
            disabled={fetching}
          >
            Save
          </button>
        </div>
      </form>
    );
  }
  return (
    <div className={classes.description}>
      {issue.description !== null ? (
        <Markdown source={issue.description} />
      ) : (
        <p className="muted">No description.</p>
      )}
      <button
        type="button"
        className={`button ${classes.edit}`}
        onClick={() => setDraft(issue.description ?? "")}
      >
        Edit
      </button>
    </div>
  );
};
