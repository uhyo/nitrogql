"use client";

import { useState } from "react";
import { useMutation } from "urql";
import CreateCommentMutation from "./createComment.graphql";
import classes from "./IssueCommentComposer.module.css";

/** Form to write a new comment, or a reply to the comment of `parentId`. */
export const IssueCommentComposer: React.FC<{
  issueId: string;
  parentId?: string;
  /** Called after the comment is posted or canceled. Only for replies. */
  onDone?: () => void;
}> = ({ issueId, parentId, onDone }) => {
  const [{ fetching, error }, createComment] = useMutation(
    CreateCommentMutation,
  );
  const [body, setBody] = useState("");
  const isReply = parentId !== undefined;
  return (
    <form
      className={classes.composer}
      onSubmit={async (e) => {
        e.preventDefault();
        if (body.trim() === "") {
          return;
        }
        const result = await createComment({
          input: { issueId, body, parentId },
        });
        if (!result.error) {
          setBody("");
          onDone?.();
        }
      }}
    >
      <textarea
        className="input"
        value={body}
        aria-label={isReply ? "Reply" : "Comment"}
        placeholder={isReply ? "Write a reply…" : "Leave a comment…"}
        autoFocus={isReply}
        onChange={(e) => setBody(e.target.value)}
      />
      {error ? <p className="muted">{error.message}</p> : null}
      <div className={classes.buttons}>
        {onDone ? (
          <button type="button" className="button" onClick={onDone}>
            Cancel
          </button>
        ) : null}
        <button
          type="submit"
          className="button"
          data-primary
          disabled={fetching || body.trim() === ""}
        >
          {isReply ? "Reply" : "Comment"}
        </button>
      </div>
    </form>
  );
};
