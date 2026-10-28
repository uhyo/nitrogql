"use client";

import { useState } from "react";
import { useMutation } from "urql";
import type { ReactionsFragment } from "./ReactionsFragment.graphql";
import ToggleReactionMutation from "./toggleReaction.graphql";
import classes from "./Reactions.module.css";

const emojis = ["👍", "🎉", "❤️", "👀", "🚀", "😄"];

/** Reactions to an issue, a comment or a project update. */
export const Reactions: React.FC<{
  subjectId: string;
  reactions: readonly ReactionsFragment[];
}> = ({ subjectId, reactions }) => {
  const [{ fetching }, toggleReaction] = useMutation(ToggleReactionMutation);
  const [pickerOpen, setPickerOpen] = useState(false);
  const toggle = (emoji: string) => {
    setPickerOpen(false);
    toggleReaction({ subjectId, emoji });
  };
  return (
    <div className={classes.reactions}>
      {reactions.map((reaction) => (
        <button
          key={reaction.emoji}
          type="button"
          className={classes.reaction}
          data-reacted={reaction.viewerHasReacted || undefined}
          disabled={fetching}
          title={reaction.users.map((user) => user.name).join(", ")}
          onClick={() => toggle(reaction.emoji)}
        >
          {reaction.emoji} {reaction.count}
        </button>
      ))}
      <span className={classes.pickerContainer}>
        <button
          type="button"
          className={classes.reaction}
          aria-label="Add reaction"
          onClick={() => setPickerOpen((open) => !open)}
        >
          ☺︎+
        </button>
        {pickerOpen ? (
          <span className={classes.picker}>
            {emojis.map((emoji) => (
              <button key={emoji} type="button" onClick={() => toggle(emoji)}>
                {emoji}
              </button>
            ))}
          </span>
        ) : null}
      </span>
    </div>
  );
};
