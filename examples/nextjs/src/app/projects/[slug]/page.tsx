"use client";

import { useParams } from "next/navigation";
import { useState } from "react";
import { useQuery } from "urql";
import {
  initialIssueFilterState,
  toIssueFilter,
} from "@/components/IssueFilters";
import { Markdown } from "@/components/Markdown";
import { MilestoneItem } from "./MilestoneItem";
import { ProjectHeader } from "./ProjectHeader";
import { ProjectIssues } from "./ProjectIssues";
import { ProjectMembers } from "./ProjectMembers";
import { ProjectUpdateCard } from "./ProjectUpdateCard";
import { ProjectUpdateComposer } from "./ProjectUpdateComposer";
import ProjectDetailQuery from "./project.graphql";
import classes from "./project.module.css";

const pageSize = 50;

export default function ProjectPage() {
  const { slug } = useParams<{ slug: string }>();
  const [filter, setFilter] = useState(initialIssueFilterState);
  const [first, setFirst] = useState(pageSize);
  const [{ data, fetching, error }] = useQuery({
    query: ProjectDetailQuery,
    variables: { slug, filter: toIssueFilter(filter), first },
  });

  if (error) {
    return (
      <div className="page">
        <p className="muted">{error.message}</p>
      </div>
    );
  }
  if (!data) {
    return (
      <div className="page">
        {fetching ? <p className="muted">Loading…</p> : null}
      </div>
    );
  }
  const { project } = data;
  if (project === null) {
    return (
      <div className="page">
        <p className="muted">Project “{slug}” was not found.</p>
      </div>
    );
  }

  return (
    <div className="page">
      <ProjectHeader project={project} />
      <div className={classes.layout}>
        <div>
          <section className="section">
            <h2>Description</h2>
            {project.description !== null ? (
              <Markdown source={project.description} />
            ) : (
              <p className="muted">No description</p>
            )}
          </section>
          <section className="section">
            <h2>Milestones</h2>
            {project.milestones.length > 0 ? (
              <ul className={classes.milestones}>
                {project.milestones.map((milestone) => (
                  <MilestoneItem
                    key={milestone.id}
                    milestone={milestone}
                    color={project.color}
                  />
                ))}
              </ul>
            ) : (
              <p className="muted">No milestones</p>
            )}
          </section>
          <section className="section">
            <h2>Updates</h2>
            <div className={classes.updates}>
              <ProjectUpdateComposer
                key={project.id}
                projectId={project.id}
                initialHealth={project.health}
              />
              {project.updates.map((update) => (
                <ProjectUpdateCard key={update.id} update={update} />
              ))}
            </div>
          </section>
          <section className="section">
            <h2>Issues</h2>
            <ProjectIssues
              issues={project.issues}
              milestones={project.milestones}
              filter={filter}
              onFilterChange={(filter) => {
                setFilter(filter);
                setFirst(pageSize);
              }}
              onLoadMore={() => setFirst((first) => first + pageSize)}
            />
          </section>
        </div>
        <aside>
          <section className="section">
            <h2>Members</h2>
            <ProjectMembers project={project} />
          </section>
        </aside>
      </div>
    </div>
  );
}
