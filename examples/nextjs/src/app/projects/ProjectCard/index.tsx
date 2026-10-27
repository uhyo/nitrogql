import Link from "next/link";
import { Avatar } from "@/components/Avatar";
import { ProgressBar } from "@/components/ProgressBar";
import { UserChip } from "@/components/UserChip";
import { formatDate } from "@/lib/format";
import { ProjectHealthBadge } from "../ProjectHealthBadge";
import type { ProjectCardFragment } from "./ProjectCardFragment.graphql";
import classes from "./ProjectCard.module.css";

const maxAvatars = 5;

/** Summary of a project in the list of projects. */
export const ProjectCard: React.FC<{ project: ProjectCardFragment }> = ({
  project,
}) => {
  const hiddenMembers = project.members.length - maxAvatars;
  return (
    <article
      className={classes.card}
      style={{ "--project-color": project.color } as React.CSSProperties}
    >
      <h3 className={classes.title}>
        <span aria-hidden>{project.icon}</span>
        <Link href={`/projects/${project.slug}`}>{project.name}</Link>
      </h3>
      {project.description !== null ? (
        <p className={`${classes.description} muted`}>{project.description}</p>
      ) : null}
      <div className={classes.meta}>
        <ProgressBar progress={project.progress} color={project.color} />
        <ProjectHealthBadge project={project} />
      </div>
      <div className={classes.meta}>
        <span title="Team">
          {project.team.icon} {project.team.name}
        </span>
        {project.lead !== null ? (
          <UserChip user={project.lead} />
        ) : (
          <span className="muted">No lead</span>
        )}
        {project.members.length > 0 ? (
          <span className={classes.members} title="Members">
            {project.members.slice(0, maxAvatars).map((member) => (
              <Avatar key={member.id} user={member} size={18} />
            ))}
            {hiddenMembers > 0 ? (
              <span className="muted">&nbsp;+{hiddenMembers}</span>
            ) : null}
          </span>
        ) : null}
      </div>
      <div className={`${classes.meta} muted`}>
        {project.targetDate !== null ? (
          <span title="Target date">🎯 {formatDate(project.targetDate)}</span>
        ) : null}
        <span>
          {project.milestones.length}{" "}
          {project.milestones.length === 1 ? "milestone" : "milestones"}
        </span>
      </div>
    </article>
  );
};
