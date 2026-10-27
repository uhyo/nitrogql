"use client";

import {
  IssueFilters,
  type IssueFilterState,
  initialIssueFilterState,
  toIssueFilter,
} from "@/components/IssueFilters";
import { priorities, priorityLabels } from "@/components/PriorityIcon";
import type {
  IssueFilter,
  IssuePriority,
  IssueSort,
  IssueSortField,
  SortDirection,
} from "@/schema/base.graphql";
import type { TeamIssueFiltersFragment } from "./TeamIssueFiltersFragment.graphql";

/** Value of the assignee select: a user ID, "unassigned" or "" (anyone). */
type AssigneeValue = string;

export type TeamIssueFilterState = IssueFilterState & {
  assignee: AssigneeValue;
  labelId: string;
  priority: IssuePriority | "";
  sort: `${IssueSortField}:${SortDirection}`;
};

export const initialTeamIssueFilterState: TeamIssueFilterState = {
  ...initialIssueFilterState,
  assignee: "",
  labelId: "",
  priority: "",
  sort: "UPDATED_AT:DESC",
};

const sortOptions: {
  value: TeamIssueFilterState["sort"];
  label: string;
}[] = [
  { value: "UPDATED_AT:DESC", label: "Recently updated" },
  { value: "CREATED_AT:DESC", label: "Newest" },
  { value: "CREATED_AT:ASC", label: "Oldest" },
  { value: "PRIORITY:DESC", label: "Priority" },
  { value: "DUE_DATE:ASC", label: "Due date" },
  { value: "NUMBER:ASC", label: "Number" },
];

/** Converts the state of filter controls to `IssueFilter` input. */
export function toTeamIssueFilter(state: TeamIssueFilterState): IssueFilter {
  return {
    ...toIssueFilter(state),
    assigneeIds:
      state.assignee === "" || state.assignee === "unassigned"
        ? null
        : [state.assignee],
    unassigned: state.assignee === "unassigned" || null,
    labelIds: state.labelId === "" ? null : [state.labelId],
    priorities: state.priority === "" ? null : [state.priority],
  };
}

export function toIssueSort(state: TeamIssueFilterState): IssueSort {
  const [field, direction] = state.sort.split(":") as [
    IssueSortField,
    SortDirection,
  ];
  return { field, direction };
}

/** Filters of the issue list of a team, including team-specific ones. */
export const TeamIssueFilters: React.FC<{
  team: TeamIssueFiltersFragment | undefined;
  value: TeamIssueFilterState;
  onChange: (value: TeamIssueFilterState) => void;
}> = ({ team, value, onChange }) => (
  <IssueFilters
    value={value}
    onChange={(filter) => onChange({ ...value, ...filter })}
  >
    <select
      className="input"
      aria-label="Assignee"
      value={value.assignee}
      onChange={(e) => onChange({ ...value, assignee: e.target.value })}
    >
      <option value="">Any assignee</option>
      <option value="unassigned">Unassigned</option>
      {team?.members.map(({ user }) => (
        <option key={user.id} value={user.id}>
          {user.name}
        </option>
      ))}
    </select>
    <select
      className="input"
      aria-label="Label"
      value={value.labelId}
      onChange={(e) => onChange({ ...value, labelId: e.target.value })}
    >
      <option value="">Any label</option>
      {team?.labels.map((label) => (
        <option key={label.id} value={label.id}>
          {label.name}
        </option>
      ))}
    </select>
    <select
      className="input"
      aria-label="Priority"
      value={value.priority}
      onChange={(e) =>
        onChange({
          ...value,
          priority: e.target.value as TeamIssueFilterState["priority"],
        })
      }
    >
      <option value="">Any priority</option>
      {priorities.map((priority) => (
        <option key={priority} value={priority}>
          {priorityLabels[priority]}
        </option>
      ))}
    </select>
  </IssueFilters>
);

/** Select of the order of issues. */
export const TeamIssueSortSelect: React.FC<{
  value: TeamIssueFilterState;
  onChange: (value: TeamIssueFilterState) => void;
}> = ({ value, onChange }) => (
  <select
    className="input"
    aria-label="Sort"
    value={value.sort}
    onChange={(e) =>
      onChange({
        ...value,
        sort: e.target.value as TeamIssueFilterState["sort"],
      })
    }
  >
    {sortOptions.map((option) => (
      <option key={option.value} value={option.value}>
        Sort: {option.label}
      </option>
    ))}
  </select>
);
