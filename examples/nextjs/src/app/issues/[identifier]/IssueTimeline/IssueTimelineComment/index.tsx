"use client";

import { useState } from "react";
import { IssueComment } from "../IssueComment";
import { IssueCommentComposer } from "../IssueCommentComposer";
import type { IssueTimelineCommentFragment } from "./IssueTimelineCommentFragment.graphql";
import classes from "./IssueTimelineComment.module.css";

/** A top-level comment in the timeline with its replies. */
export const IssueTimelineComment: React.FC<{
  comment: IssueTimelineCommentFragment;
  issueId: string;
}> = ({ comment, issueId }) => {
  const [replying, setReplying] = useState(false);
  return (
    <div className={classes.thread}>
      <IssueComment
        comment={comment}
        actions={
          <button type="button" onClick={() => setReplying(true)}>
            Reply
          </button>
        }
      />
      {comment.replies.length > 0 || replying ? (
        <ol className={classes.replies}>
          {comment.replies.map((reply) => (
            <li key={reply.id}>
              <IssueComment comment={reply} />
            </li>
          ))}
          {replying ? (
            <li>
              <IssueCommentComposer
                issueId={issueId}
                parentId={comment.id}
                onDone={() => setReplying(false)}
              />
            </li>
          ) : null}
        </ol>
      ) : null}
    </div>
  );
};
