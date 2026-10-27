"use client";

import type { IssueFilter, WorkflowStateType } from "@/schema/base.graphql";
import classes from "./IssueFilters.module.css";

export type StateTab = "active" | "backlog" | "done" | "all";

const stateTabs: { value: StateTab; label: string }[] = [
  { value: "active", label: "Active" },
  { value: "backlog", label: "Backlog" },
  { value: "done", label: "Done" },
  { value: "all", label: "All" },
];

const stateTypesOfTab: Record<StateTab, WorkflowStateType[] | null> = {
  active: ["UNSTARTED", "STARTED"],
  backlog: ["TRIAGE", "BACKLOG"],
  done: ["COMPLETED", "CANCELED"],
  all: null,
};

export type IssueFilterState = {
  tab: StateTab;
  search: string;
};

export const initialIssueFilterState: IssueFilterState = {
  tab: "active",
  search: "",
};

/** Converts the state of filter controls to `IssueFilter` input. */
export function toIssueFilter(state: IssueFilterState): IssueFilter {
  return {
    stateTypes: stateTypesOfTab[state.tab],
    search: state.search.trim() || null,
  };
}

/** Controls for filtering issues by state and text. */
export const IssueFilters: React.FC<{
  value: IssueFilterState;
  onChange: (value: IssueFilterState) => void;
  children?: React.ReactNode;
}> = ({ value, onChange, children }) => (
  <div className={classes.filters}>
    <div role="tablist" className={classes.tabs}>
      {stateTabs.map((tab) => (
        <button
          key={tab.value}
          type="button"
          role="tab"
          aria-selected={value.tab === tab.value}
          className={classes.tab}
          onClick={() => onChange({ ...value, tab: tab.value })}
        >
          {tab.label}
        </button>
      ))}
    </div>
    {children}
    <input
      type="search"
      className="input"
      placeholder="Filter by text…"
      value={value.search}
      onChange={(e) => onChange({ ...value, search: e.target.value })}
    />
  </div>
);
