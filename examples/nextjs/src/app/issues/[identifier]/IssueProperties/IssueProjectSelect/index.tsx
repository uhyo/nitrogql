import { ProjectChip } from "@/components/ProjectChip";
import { formatDate } from "@/lib/format";
import type { UpdateIssueInput } from "@/schema/base.graphql";
import type { IssueProjectSelectFragment } from "./IssueProjectSelectFragment.graphql";

const none = "";

/**
 * Selects a project of the issue's team, and a milestone of the project.
 * Renders `<dt>` and `<dd>` pairs for both.
 */
export const IssueProjectSelect: React.FC<{
  issue: IssueProjectSelectFragment;
  disabled?: boolean;
  onChange: (input: UpdateIssueInput) => void;
}> = ({ issue, disabled, onChange }) => {
  const { project, milestone } = issue;
  // Only active projects are listed, but the current project is kept
  const projects =
    project === null || issue.team.projects.some((p) => p.id === project.id)
      ? issue.team.projects
      : [project, ...issue.team.projects];
  return (
    <>
      <dt>Project</dt>
      <dd>
        <select
          className="input"
          aria-label="Project"
          value={project?.id ?? none}
          disabled={disabled}
          onChange={(e) =>
            onChange({
              projectId: e.target.value === none ? null : e.target.value,
            })
          }
        >
          <option value={none}>No project</option>
          {projects.map((project) => (
            <option key={project.id} value={project.id}>
              {`${project.icon} ${project.name}`}
            </option>
          ))}
        </select>
      </dd>
      {project !== null ? (
        <>
          <dt />
          <dd>
            <ProjectChip project={project} />
          </dd>
          <dt>Milestone</dt>
          <dd>
            <select
              className="input"
              aria-label="Milestone"
              value={milestone?.id ?? none}
              disabled={disabled || project.milestones.length === 0}
              onChange={(e) =>
                onChange({
                  milestoneId: e.target.value === none ? null : e.target.value,
                })
              }
            >
              <option value={none}>No milestone</option>
              {project.milestones.map((milestone) => (
                <option key={milestone.id} value={milestone.id}>
                  {milestone.targetDate !== null
                    ? `${milestone.name} (${formatDate(milestone.targetDate)})`
                    : milestone.name}
                </option>
              ))}
            </select>
          </dd>
        </>
      ) : null}
    </>
  );
};
