"use client";

import { useState } from "react";
import { useQuery } from "urql";
import type { ProjectStatus } from "@/schema/base.graphql";
import { ProjectCard } from "./ProjectCard";
import {
  ProjectStatusIcon,
  projectStatusLabels,
  projectStatuses,
} from "./ProjectStatusIcon";
import ProjectsPageQuery from "./projects.graphql";
import classes from "./projects.module.css";

export default function ProjectsPage() {
  const [teamKey, setTeamKey] = useState<string | null>(null);
  const [status, setStatus] = useState<ProjectStatus | null>(null);
  const [{ data, fetching, error }] = useQuery({
    query: ProjectsPageQuery,
    variables: {
      teamKey,
      status: status === null ? null : [status],
    },
  });

  const groups = projectStatuses
    .map((status) => ({
      status,
      projects: (data?.projects ?? []).filter(
        (project) => project.status === status,
      ),
    }))
    .filter((group) => group.projects.length > 0);

  return (
    <div className="page">
      <header className="pageHeader">
        <h1>Projects</h1>
        <div className={classes.filters}>
          <select
            className="input"
            value={teamKey ?? ""}
            aria-label="Team"
            onChange={(e) => setTeamKey(e.target.value || null)}
          >
            <option value="">All teams</option>
            {data?.teams.map((team) => (
              <option key={team.id} value={team.key}>
                {team.icon} {team.name}
              </option>
            ))}
          </select>
          <select
            className="input"
            value={status ?? ""}
            aria-label="Status"
            onChange={(e) =>
              setStatus((e.target.value || null) as ProjectStatus | null)
            }
          >
            <option value="">All statuses</option>
            {projectStatuses.map((status) => (
              <option key={status} value={status}>
                {projectStatusLabels[status]}
              </option>
            ))}
          </select>
        </div>
      </header>
      {error ? <p className="muted">{error.message}</p> : null}
      {data ? (
        groups.length > 0 ? (
          groups.map(({ status, projects }) => (
            <section key={status} className="section">
              <h2 className={classes.groupHeader}>
                <ProjectStatusIcon status={status} showName />
                <span className="muted">{projects.length}</span>
              </h2>
              <div className={classes.grid}>
                {projects.map((project) => (
                  <ProjectCard key={project.id} project={project} />
                ))}
              </div>
            </section>
          ))
        ) : (
          <p className="muted">No projects</p>
        )
      ) : fetching ? (
        <p className="muted">Loading…</p>
      ) : null}
    </div>
  );
}
