import type { ProjectHealth } from "@/schema/base.graphql";
import type { ProjectHealthBadgeFragment } from "./ProjectHealthBadgeFragment.graphql";
import classes from "./ProjectHealthBadge.module.css";

export const healthLabels: Record<ProjectHealth, string> = {
  ON_TRACK: "On track",
  AT_RISK: "At risk",
  OFF_TRACK: "Off track",
};

export const healths: readonly ProjectHealth[] = [
  "ON_TRACK",
  "AT_RISK",
  "OFF_TRACK",
];

const healthColors: Record<ProjectHealth, string> = {
  ON_TRACK: "#26b5ce",
  AT_RISK: "#f2994a",
  OFF_TRACK: "#eb5757",
};

/** Health of a project, as reported by its latest update. */
export const ProjectHealthBadge: React.FC<{
  project: ProjectHealthBadgeFragment;
}> = ({ project }) =>
  project.health !== null ? (
    <HealthBadge health={project.health} />
  ) : (
    <span className={`${classes.badge} muted`}>No updates</span>
  );

/** Badge of a health value, also used for health of each project update. */
export const HealthBadge: React.FC<{ health: ProjectHealth }> = ({
  health,
}) => (
  <span className={classes.badge}>
    <span
      className={classes.dot}
      style={{ background: healthColors[health] }}
    />
    {healthLabels[health]}
  </span>
);
