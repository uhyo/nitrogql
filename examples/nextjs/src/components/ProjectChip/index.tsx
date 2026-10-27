import Link from "next/link";
import type { ProjectChipFragment } from "./ProjectChipFragment.graphql";

export const ProjectChip: React.FC<{ project: ProjectChipFragment }> = ({
  project,
}) => (
  <Link
    href={`/projects/${project.slug}`}
    style={{
      display: "inline-flex",
      alignItems: "center",
      gap: "0.3em",
      whiteSpace: "nowrap",
      borderBottom: `2px solid ${project.color}`,
    }}
  >
    <span aria-hidden>{project.icon}</span>
    {project.name}
  </Link>
);
