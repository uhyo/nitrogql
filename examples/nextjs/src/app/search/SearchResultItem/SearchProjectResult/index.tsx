import { Avatar } from "@/components/Avatar";
import { ProgressBar } from "@/components/ProgressBar";
import { ProjectChip } from "@/components/ProjectChip";
import { formatDate } from "@/lib/format";
import type { ProjectStatus } from "@/schema/base.graphql";
import type { SearchProjectResultFragment } from "./SearchProjectResultFragment.graphql";
import classes from "../SearchResultItem.module.css";

const statusLabels: Record<ProjectStatus, string> = {
  PLANNED: "Planned",
  IN_PROGRESS: "In progress",
  PAUSED: "Paused",
  COMPLETED: "Completed",
  CANCELED: "Canceled",
};

export const SearchProjectResult: React.FC<{
  project: SearchProjectResultFragment;
}> = ({ project }) => (
  <div className={classes.row}>
    <ProjectChip project={project} />
    <span className={`${classes.description} muted`}>
      {project.description}
    </span>
    <span className={classes.meta}>
      <span className="muted">{project.team.name}</span>
      <span>{statusLabels[project.projectStatus]}</span>
      <ProgressBar progress={project.progress} color={project.color} />
      {project.targetDate !== null ? (
        <span className="muted" title="Target date">
          🎯 {formatDate(project.targetDate)}
        </span>
      ) : null}
      {project.lead !== null ? <Avatar user={project.lead} /> : null}
    </span>
  </div>
);
