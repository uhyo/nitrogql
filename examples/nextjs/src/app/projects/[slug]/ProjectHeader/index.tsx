"use client";

import Link from "next/link";
import { useMutation } from "urql";
import { ProgressBar } from "@/components/ProgressBar";
import { UserChip } from "@/components/UserChip";
import { formatDate } from "@/lib/format";
import type { ProjectStatus } from "@/schema/base.graphql";
import { ProjectHealthBadge } from "../../ProjectHealthBadge";
import {
  ProjectStatusIcon,
  projectStatusLabels,
  projectStatuses,
} from "../../ProjectStatusIcon";
import { ProjectLeadSelect } from "../ProjectLeadSelect";
import type { ProjectHeaderFragment } from "./ProjectHeaderFragment.graphql";
import UpdateProjectStatusMutation from "./updateProjectStatus.graphql";
import classes from "./ProjectHeader.module.css";

export const ProjectHeader: React.FC<{ project: ProjectHeaderFragment }> = ({
  project,
}) => {
  const [{ fetching, error }, updateStatus] = useMutation(
    UpdateProjectStatusMutation,
  );
  return (
    <header
      className={classes.header}
      style={{ "--project-color": project.color } as React.CSSProperties}
    >
      <p className="muted">
        <Link href="/projects">Projects</Link> ›{" "}
        <Link href={`/teams/${project.team.key}/issues`}>
          {project.team.icon} {project.team.name}
        </Link>
      </p>
      <h1 className={classes.title}>
        <span className={classes.icon} aria-hidden>
          {project.icon}
        </span>
        {project.name}
      </h1>
      <dl className={classes.properties}>
        <dt>Status</dt>
        <dd>
          <ProjectStatusIcon status={project.status} />
          <select
            className="input"
            aria-label="Status"
            value={project.status}
            disabled={fetching}
            onChange={(e) =>
              updateStatus({
                id: project.id,
                status: e.target.value as ProjectStatus,
              })
            }
          >
            {projectStatuses.map((status) => (
              <option key={status} value={status}>
                {projectStatusLabels[status]}
              </option>
            ))}
          </select>
          {error ? <span className="muted">{error.message}</span> : null}
        </dd>
        <dt>Health</dt>
        <dd>
          <ProjectHealthBadge project={project} />
        </dd>
        <dt>Progress</dt>
        <dd>
          <ProgressBar progress={project.progress} color={project.color} />
        </dd>
        <dt>Lead</dt>
        <dd>
          {project.lead !== null ? <UserChip user={project.lead} /> : null}
          <ProjectLeadSelect project={project} />
        </dd>
        <dt>Dates</dt>
        <dd>
          {project.startDate !== null
            ? formatDate(project.startDate)
            : "Not started"}
          {" → "}
          {project.targetDate !== null
            ? formatDate(project.targetDate)
            : "No target date"}
        </dd>
      </dl>
    </header>
  );
};
