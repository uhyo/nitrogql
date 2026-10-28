"use client";

import { useMutation } from "urql";
import {
  PriorityIcon,
  priorities,
  priorityLabels,
} from "@/components/PriorityIcon";
import { RelativeTime } from "@/components/RelativeTime";
import { UserChip } from "@/components/UserChip";
import type { IssuePriority, UpdateIssueInput } from "@/schema/base.graphql";
import { IssueAssigneeSelect } from "./IssueAssigneeSelect";
import { IssueCycleSelect } from "./IssueCycleSelect";
import { IssueLabelsEditor } from "./IssueLabelsEditor";
import { IssueProjectSelect } from "./IssueProjectSelect";
import { IssueStateSelect } from "./IssueStateSelect";
import type { IssuePropertiesFragment } from "./IssuePropertiesFragment.graphql";
import UpdateIssuePropertiesMutation from "./updateIssueProperties.graphql";
import classes from "./IssueProperties.module.css";

/** Panel of editable properties of an issue. */
export const IssueProperties: React.FC<{ issue: IssuePropertiesFragment }> = ({
  issue,
}) => {
  const [{ fetching, error }, updateIssue] = useMutation(
    UpdateIssuePropertiesMutation,
  );
  const update = (input: UpdateIssueInput) => {
    updateIssue({ id: issue.id, input });
  };
  return (
    <dl className={classes.properties}>
      <dt>State</dt>
      <dd>
        <IssueStateSelect issue={issue} disabled={fetching} onChange={update} />
      </dd>
      <dt>Assignee</dt>
      <dd>
        <IssueAssigneeSelect
          issue={issue}
          disabled={fetching}
          onChange={update}
        />
      </dd>
      <dt>Priority</dt>
      <dd>
        <PriorityIcon priority={issue.priority} />
        <select
          className="input"
          aria-label="Priority"
          value={issue.priority}
          disabled={fetching}
          onChange={(e) =>
            update({ priority: e.target.value as IssuePriority })
          }
        >
          {priorities.map((priority) => (
            <option key={priority} value={priority}>
              {priorityLabels[priority]}
            </option>
          ))}
        </select>
      </dd>
      <dt>Labels</dt>
      <dd>
        <IssueLabelsEditor issue={issue} />
      </dd>
      <IssueProjectSelect issue={issue} disabled={fetching} onChange={update} />
      <dt>Cycle</dt>
      <dd>
        <IssueCycleSelect issue={issue} disabled={fetching} onChange={update} />
      </dd>
      <dt>Estimate</dt>
      <dd>
        <input
          // Reset the input when the value is updated
          key={issue.estimate}
          type="number"
          className="input"
          aria-label="Estimate"
          min={0}
          placeholder="No estimate"
          defaultValue={issue.estimate ?? ""}
          onBlur={(e) => {
            const estimate =
              e.target.value === "" ? null : Number(e.target.value);
            if (estimate !== issue.estimate) {
              update({ estimate });
            }
          }}
        />
        <span className="muted">pt</span>
      </dd>
      <dt>Due date</dt>
      <dd>
        <input
          type="date"
          className="input"
          aria-label="Due date"
          value={issue.dueDate ?? ""}
          disabled={fetching}
          onChange={(e) => update({ dueDate: e.target.value || null })}
        />
      </dd>
      <hr className={classes.separator} />
      <dt>Creator</dt>
      <dd>
        <UserChip user={issue.creator} />
      </dd>
      <dt>Created</dt>
      <dd>
        <RelativeTime date={issue.createdAt} />
      </dd>
      <dt>Updated</dt>
      <dd>
        <RelativeTime date={issue.updatedAt} />
      </dd>
      {issue.completedAt !== null ? (
        <>
          <dt>Completed</dt>
          <dd>
            <RelativeTime date={issue.completedAt} />
          </dd>
        </>
      ) : null}
      {issue.canceledAt !== null ? (
        <>
          <dt>Canceled</dt>
          <dd>
            <RelativeTime date={issue.canceledAt} />
          </dd>
        </>
      ) : null}
      {error ? (
        <dd className={`${classes.error} muted`}>{error.message}</dd>
      ) : null}
    </dl>
  );
};
