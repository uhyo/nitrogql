"use client";

import { useMutation } from "urql";
import { LabelBadge } from "@/components/LabelBadge";
import type { IssueLabelsEditorFragment } from "./IssueLabelsEditorFragment.graphql";
import AddIssueLabelMutation from "./addIssueLabel.graphql";
import RemoveIssueLabelMutation from "./removeIssueLabel.graphql";
import classes from "./IssueLabelsEditor.module.css";

/** Labels of an issue, with controls to add and remove labels. */
export const IssueLabelsEditor: React.FC<{
  issue: IssueLabelsEditorFragment;
}> = ({ issue }) => {
  const [{ fetching: adding }, addLabel] = useMutation(AddIssueLabelMutation);
  const [{ fetching: removing }, removeLabel] = useMutation(
    RemoveIssueLabelMutation,
  );
  const labelIds = new Set(issue.labels.map((label) => label.id));
  const addableLabels = issue.team.labels.filter(
    (label) => !labelIds.has(label.id),
  );
  return (
    <div className={classes.labels}>
      {issue.labels.map((label) => (
        <LabelBadge
          key={label.id}
          label={label}
          onRemove={
            removing
              ? undefined
              : () => removeLabel({ issueId: issue.id, labelId: label.id })
          }
        />
      ))}
      {addableLabels.length > 0 ? (
        <select
          className={`input ${classes.add}`}
          aria-label="Add label"
          value=""
          disabled={adding}
          onChange={(e) => {
            if (e.target.value !== "") {
              addLabel({ issueId: issue.id, labelId: e.target.value });
            }
          }}
        >
          <option value="">+ Label</option>
          {addableLabels.map((label) => (
            <option key={label.id} value={label.id}>
              {label.name}
            </option>
          ))}
        </select>
      ) : null}
    </div>
  );
};
