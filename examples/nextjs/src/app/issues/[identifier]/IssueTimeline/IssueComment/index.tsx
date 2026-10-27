"use client";

import { useState } from "react";
import { useMutation } from "urql";
import { Markdown } from "@/components/Markdown";
import { Reactions } from "@/components/Reactions";
import { RelativeTime } from "@/components/RelativeTime";
import { UserChip } from "@/components/UserChip";
import { formatDateTime } from "@/lib/format";
import type { IssueCommentFragment } from "./IssueCommentFragment.graphql";
import DeleteCommentMutation from "./deleteComment.graphql";
import UpdateCommentMutation from "./updateComment.graphql";
import classes from "./IssueComment.module.css";

/** A comment, which can be edited and deleted by its author. */
export const IssueComment: React.FC<{
  comment: IssueCommentFragment;
  /** Additional actions shown in the header, such as "Reply". */
  actions?: React.ReactNode;
}> = ({ comment, actions }) => {
  const [{ fetching: updating, error: updateError }, updateComment] =
    useMutation(UpdateCommentMutation);
  const [{ fetching: deleting }, deleteComment] = useMutation(
    DeleteCommentMutation,
  );
  const [draft, setDraft] = useState<string | null>(null);

  return (
    <article className={classes.comment}>
      <header className={classes.header}>
        <UserChip user={comment.author} />
        <RelativeTime date={comment.createdAt} className="muted" />
        {comment.editedAt !== null ? (
          <span className="muted" title={formatDateTime(comment.editedAt)}>
            (edited)
          </span>
        ) : null}
        <span className={classes.actions}>
          {actions}
          {comment.viewerCanEdit && draft === null ? (
            <>
              <button type="button" onClick={() => setDraft(comment.body)}>
                Edit
              </button>
              <button
                type="button"
                disabled={deleting}
                onClick={() => {
                  if (confirm("Delete this comment?")) {
                    deleteComment({ id: comment.id });
                  }
                }}
              >
                Delete
              </button>
            </>
          ) : null}
        </span>
      </header>
      {draft !== null ? (
        <form
          className={classes.editor}
          onSubmit={async (e) => {
            e.preventDefault();
            if (draft.trim() === "") {
              return;
            }
            const result = await updateComment({ id: comment.id, body: draft });
            if (!result.error) {
              setDraft(null);
            }
          }}
        >
          <textarea
            className="input"
            value={draft}
            aria-label="Edit comment"
            autoFocus
            onChange={(e) => setDraft(e.target.value)}
          />
          {updateError ? <p className="muted">{updateError.message}</p> : null}
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
              disabled={updating}
            >
              Save
            </button>
          </div>
        </form>
      ) : (
        <Markdown source={comment.body} />
      )}
      <Reactions subjectId={comment.id} reactions={comment.reactions} />
    </article>
  );
};
