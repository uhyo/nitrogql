import type { ProjectStatus } from "@/schema/base.graphql";

export const projectStatusLabels: Record<ProjectStatus, string> = {
  PLANNED: "Planned",
  IN_PROGRESS: "In progress",
  PAUSED: "Paused",
  COMPLETED: "Completed",
  CANCELED: "Canceled",
};

/** In display order. */
export const projectStatuses: readonly ProjectStatus[] = [
  "IN_PROGRESS",
  "PLANNED",
  "PAUSED",
  "COMPLETED",
  "CANCELED",
];

const statusColors: Record<ProjectStatus, string> = {
  PLANNED: "#8b8e98",
  IN_PROGRESS: "#f2c94c",
  PAUSED: "#bb87fc",
  COMPLETED: "#5e6ad2",
  CANCELED: "#95a2b3",
};

/** Icon of a project status. With `showName`, the name of the status is shown too. */
export const ProjectStatusIcon: React.FC<{
  status: ProjectStatus;
  showName?: boolean;
}> = ({ status, showName = false }) => (
  <span
    title={projectStatusLabels[status]}
    style={{ display: "inline-flex", alignItems: "center", gap: "0.4em" }}
  >
    <svg width="14" height="14" viewBox="0 0 14 14" aria-hidden>
      <rect
        x="1.5"
        y="1.5"
        width="11"
        height="11"
        rx="3"
        fill={status === "COMPLETED" ? statusColors[status] : "none"}
        stroke={statusColors[status]}
        strokeWidth="1.5"
        strokeDasharray={status === "PLANNED" ? "2 2" : undefined}
      />
      {status === "IN_PROGRESS" ? (
        <rect x="4" y="4" width="3" height="6" fill={statusColors[status]} />
      ) : status === "PAUSED" ? (
        <path d="M5.5 4.5v5M8.5 4.5v5" stroke={statusColors[status]} />
      ) : status === "CANCELED" ? (
        <path d="M5 5l4 4M9 5l-4 4" stroke={statusColors[status]} />
      ) : null}
    </svg>
    {showName ? <span>{projectStatusLabels[status]}</span> : null}
  </span>
);
