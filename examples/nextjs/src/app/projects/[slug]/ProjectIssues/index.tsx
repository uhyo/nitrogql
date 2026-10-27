"use client";

import { useState } from "react";
import { IssueFilters, type IssueFilterState } from "@/components/IssueFilters";
import { IssueList } from "@/components/IssueList";
import type {
  ProjectIssues_issues,
  ProjectIssues_milestones,
} from "./ProjectIssuesFragment.graphql";

/** Value of the milestone filter: a milestone ID, issues without milestone, or all. */
type MilestoneFilter = string | "none" | "all";

/**
 * Issues of a project. The state and text filters are applied by the server,
 * while the milestone filter is applied to the loaded issues.
 */
export const ProjectIssues: React.FC<{
  issues: ProjectIssues_issues;
  milestones: readonly ProjectIssues_milestones[];
  filter: IssueFilterState;
  onFilterChange: (filter: IssueFilterState) => void;
  onLoadMore: () => void;
}> = ({ issues, milestones, filter, onFilterChange, onLoadMore }) => {
  const [milestone, setMilestone] = useState<MilestoneFilter>("all");
  const nodes =
    milestone === "all"
      ? issues.nodes
      : issues.nodes.filter((issue) =>
          milestone === "none"
            ? issue.milestone === null
            : issue.milestone?.id === milestone,
        );
  return (
    <>
      <IssueFilters value={filter} onChange={onFilterChange}>
        <select
          className="input"
          aria-label="Milestone"
          value={milestone}
          onChange={(e) => setMilestone(e.target.value)}
        >
          <option value="all">All milestones</option>
          {milestones.map((milestone) => (
            <option key={milestone.id} value={milestone.id}>
              {milestone.name}
            </option>
          ))}
          <option value="none">No milestone</option>
        </select>
      </IssueFilters>
      <IssueList
        issues={{ ...issues, nodes }}
        groupByState
        hideProject
        emptyMessage="No issues in this project"
        onLoadMore={onLoadMore}
      />
    </>
  );
};
