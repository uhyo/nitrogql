"use client";

import { useState } from "react";
import { useMutation } from "urql";
import type { UserStatusInput } from "@/schema/base.graphql";
import type { UserStatusEditorFragment } from "./UserStatusEditorFragment.graphql";
import UpdateViewerStatusMutation from "./updateViewerStatus.graphql";
import classes from "./UserStatusEditor.module.css";

const emojis = ["💬", "🌴", "🤒", "🎯", "🚆", "🏠", "📅", "🎧"];

type Expiry = "never" | "30m" | "4h" | "today" | "week";

const expiryOptions: { value: Expiry; label: string }[] = [
  { value: "never", label: "Don't clear" },
  { value: "30m", label: "30 minutes" },
  { value: "4h", label: "4 hours" },
  { value: "today", label: "Today" },
  { value: "week", label: "This week" },
];

/** Converts an expiry option to a date-time, relative to `now`. */
function expiresAtOf(expiry: Expiry, now = new Date()): string | null {
  const date = new Date(now);
  switch (expiry) {
    case "never":
      return null;
    case "30m":
      date.setMinutes(date.getMinutes() + 30);
      break;
    case "4h":
      date.setHours(date.getHours() + 4);
      break;
    case "today":
      date.setHours(23, 59, 59, 0);
      break;
    case "week":
      // Until the end of Sunday
      date.setDate(date.getDate() + ((7 - date.getDay()) % 7));
      date.setHours(23, 59, 59, 0);
      break;
  }
  return date.toISOString();
}

/** Lets the viewer set or clear their status. */
export const UserStatusEditor: React.FC<{ user: UserStatusEditorFragment }> = ({
  user,
}) => {
  const [editing, setEditing] = useState(false);
  const [emoji, setEmoji] = useState(user.status?.emoji ?? emojis[0]);
  const [message, setMessage] = useState(user.status?.message ?? "");
  const [expiry, setExpiry] = useState<Expiry>("never");
  const [{ fetching, error }, updateStatus] = useMutation(
    UpdateViewerStatusMutation,
  );

  const save = async (input: UserStatusInput | null) => {
    const result = await updateStatus({ input });
    if (result.error === undefined) {
      setEditing(false);
    }
  };

  if (!editing) {
    return (
      <span className={classes.actions}>
        <button
          type="button"
          className="button"
          onClick={() => {
            setEmoji(user.status?.emoji ?? emojis[0]);
            setMessage(user.status?.message ?? "");
            setExpiry("never");
            setEditing(true);
          }}
        >
          {user.status !== null ? "Edit status" : "Set status"}
        </button>
        {user.status !== null ? (
          <button
            type="button"
            className="button"
            disabled={fetching}
            onClick={() => save(null)}
          >
            Clear status
          </button>
        ) : null}
      </span>
    );
  }

  return (
    <form
      className={classes.editor}
      onSubmit={(e) => {
        e.preventDefault();
        save({
          emoji,
          message: message.trim(),
          expiresAt: expiresAtOf(expiry),
        });
      }}
    >
      <div className={classes.emojis} role="radiogroup" aria-label="Emoji">
        {emojis.map((option) => (
          <button
            key={option}
            type="button"
            role="radio"
            aria-checked={option === emoji}
            className={classes.emoji}
            onClick={() => setEmoji(option)}
          >
            {option}
          </button>
        ))}
      </div>
      <input
        className="input"
        value={message}
        placeholder="What's your status?"
        aria-label="Status message"
        required
        autoFocus
        onChange={(e) => setMessage(e.target.value)}
      />
      <select
        className="input"
        value={expiry}
        aria-label="Clear status after"
        onChange={(e) => setExpiry(e.target.value as Expiry)}
      >
        {expiryOptions.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
      <button
        type="submit"
        className="button"
        data-primary
        disabled={fetching || message.trim() === ""}
      >
        Save
      </button>
      <button
        type="button"
        className="button"
        onClick={() => setEditing(false)}
      >
        Cancel
      </button>
      {error ? <p className="muted">{error.message}</p> : null}
    </form>
  );
};
