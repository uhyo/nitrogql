"use client";

import { useState } from "react";
import { useMutation } from "urql";
import type { ProjectHealth } from "@/schema/base.graphql";
import { healthLabels, healths } from "../../ProjectHealthBadge";
import CreateProjectUpdateMutation from "./createProjectUpdate.graphql";
import classes from "./ProjectUpdateComposer.module.css";

/** Form for posting a status update of a project. */
export const ProjectUpdateComposer: React.FC<{
  projectId: string;
  /** Health of the latest update, used as the initial value. */
  initialHealth: ProjectHealth | null;
}> = ({ projectId, initialHealth }) => {
  const [body, setBody] = useState("");
  const [health, setHealth] = useState<ProjectHealth>(
    initialHealth ?? "ON_TRACK",
  );
  const [{ fetching, error }, createUpdate] = useMutation(
    CreateProjectUpdateMutation,
  );
  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    const result = await createUpdate({
      input: { projectId, body: body.trim(), health },
    });
    if (result.data) {
      setBody("");
    }
  };
  return (
    <form className={classes.composer} onSubmit={submit}>
      <textarea
        className="input"
        placeholder="Write a project update…"
        aria-label="Update"
        value={body}
        onChange={(e) => setBody(e.target.value)}
      />
      <div className={classes.actions}>
        {error ? <span className={classes.error}>{error.message}</span> : null}
        <select
          className="input"
          aria-label="Health"
          value={health}
          onChange={(e) => setHealth(e.target.value as ProjectHealth)}
        >
          {healths.map((health) => (
            <option key={health} value={health}>
              {healthLabels[health]}
            </option>
          ))}
        </select>
        <button
          type="submit"
          className="button"
          data-primary
          disabled={fetching || body.trim() === ""}
        >
          Post update
        </button>
      </div>
    </form>
  );
};
