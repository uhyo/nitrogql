"use client";

import Link from "next/link";
import { useState } from "react";
import { useMutation } from "urql";
import { LabelBadge } from "@/components/LabelBadge";
import { priorities, priorityLabels } from "@/components/PriorityIcon";
import type { IssuePriority } from "@/schema/base.graphql";
import type { NewIssueFormFragment } from "./NewIssueFormFragment.graphql";
import CreateIssueMutation from "./createIssue.graphql";
import classes from "./NewIssueForm.module.css";

type FormState = {
  title: string;
  description: string;
  stateId: string;
  priority: IssuePriority;
  assigneeId: string;
  labelIds: readonly string[];
  projectId: string;
  cycleId: string;
  estimate: string;
  dueDate: string;
};

const initialFormState: FormState = {
  title: "",
  description: "",
  stateId: "",
  priority: "NO_PRIORITY",
  assigneeId: "",
  labelIds: [],
  projectId: "",
  cycleId: "",
  estimate: "",
  dueDate: "",
};

/** Form to create an issue in a team. Options depend on the team. */
export const NewIssueForm: React.FC<{
  team: NewIssueFormFragment;
  onClose: () => void;
}> = ({ team, onClose }) => {
  const [form, setForm] = useState(initialFormState);
  const [{ data, fetching, error }, createIssue] =
    useMutation(CreateIssueMutation);
  const update = <K extends keyof FormState>(key: K, value: FormState[K]) =>
    setForm((form) => ({ ...form, [key]: value }));
  const toggleLabel = (labelId: string) =>
    update(
      "labelIds",
      form.labelIds.includes(labelId)
        ? form.labelIds.filter((id) => id !== labelId)
        : [...form.labelIds, labelId],
    );

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    const result = await createIssue({
      input: {
        teamId: team.id,
        title: form.title.trim(),
        description: form.description.trim() || null,
        stateId: form.stateId || null,
        priority: form.priority,
        assigneeId: form.assigneeId || null,
        labelIds: form.labelIds,
        projectId: form.projectId || null,
        cycleId: form.cycleId || null,
        estimate: form.estimate === "" ? null : Number(form.estimate),
        dueDate: form.dueDate || null,
      },
    });
    if (!result.error) {
      setForm(initialFormState);
    }
  };

  return (
    <form className={classes.form} onSubmit={submit}>
      <input
        className={`input ${classes.title}`}
        placeholder="Issue title"
        aria-label="Title"
        required
        autoFocus
        value={form.title}
        onChange={(e) => update("title", e.target.value)}
      />
      <textarea
        className="input"
        placeholder="Add description…"
        aria-label="Description"
        rows={3}
        value={form.description}
        onChange={(e) => update("description", e.target.value)}
      />
      <div className={classes.options}>
        <select
          className="input"
          aria-label="State"
          value={form.stateId}
          onChange={(e) => update("stateId", e.target.value)}
        >
          <option value="">Default state</option>
          {team.workflowStates.map((state) => (
            <option key={state.id} value={state.id}>
              {state.name}
            </option>
          ))}
        </select>
        <select
          className="input"
          aria-label="Priority"
          value={form.priority}
          onChange={(e) => update("priority", e.target.value as IssuePriority)}
        >
          {priorities.map((priority) => (
            <option key={priority} value={priority}>
              {priorityLabels[priority]}
            </option>
          ))}
        </select>
        <select
          className="input"
          aria-label="Assignee"
          value={form.assigneeId}
          onChange={(e) => update("assigneeId", e.target.value)}
        >
          <option value="">No assignee</option>
          {team.members
            .filter(({ user }) => !user.deactivated)
            .map(({ user }) => (
              <option key={user.id} value={user.id}>
                {user.name}
              </option>
            ))}
        </select>
        <select
          className="input"
          aria-label="Project"
          value={form.projectId}
          onChange={(e) => update("projectId", e.target.value)}
        >
          <option value="">No project</option>
          {team.projects.map((project) => (
            <option key={project.id} value={project.id}>
              {project.icon} {project.name}
            </option>
          ))}
        </select>
        {team.cycles.length > 0 ? (
          <select
            className="input"
            aria-label="Cycle"
            value={form.cycleId}
            onChange={(e) => update("cycleId", e.target.value)}
          >
            <option value="">No cycle</option>
            {team.cycles.map((cycle) => (
              <option key={cycle.id} value={cycle.id}>
                {cycle.name ?? `Cycle ${cycle.number}`}
              </option>
            ))}
          </select>
        ) : null}
        <input
          type="number"
          className={`input ${classes.estimate}`}
          placeholder="Estimate"
          aria-label="Estimate"
          min={0}
          value={form.estimate}
          onChange={(e) => update("estimate", e.target.value)}
        />
        <input
          type="date"
          className="input"
          aria-label="Due date"
          value={form.dueDate}
          onChange={(e) => update("dueDate", e.target.value)}
        />
      </div>
      <fieldset className={classes.labels}>
        <legend className="muted">Labels</legend>
        {team.labels.map((label) => (
          <label key={label.id} className={classes.label}>
            <input
              type="checkbox"
              checked={form.labelIds.includes(label.id)}
              onChange={() => toggleLabel(label.id)}
            />
            <LabelBadge label={label} />
          </label>
        ))}
      </fieldset>
      <footer className={classes.footer}>
        {error ? (
          <span className={classes.error}>{error.message}</span>
        ) : data ? (
          <span className="muted">
            Created{" "}
            <Link href={`/issues/${data.createIssue.identifier}`}>
              {data.createIssue.identifier}
            </Link>
          </span>
        ) : null}
        <button type="button" className="button" onClick={onClose}>
          Cancel
        </button>
        <button
          type="submit"
          className="button"
          data-primary
          disabled={fetching || form.title.trim() === ""}
        >
          Create issue
        </button>
      </footer>
    </form>
  );
};
