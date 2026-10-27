import { ProjectChip } from "@/components/ProjectChip";
import type { ProjectHealth } from "@/schema/base.graphql";
import { excerpt } from "../InboxCommentExcerpt";
import type { ProjectUpdateNotificationFragment } from "./ProjectUpdateNotificationFragment.graphql";
import classes from "./ProjectUpdateNotification.module.css";

const healthLabels: Record<ProjectHealth, string> = {
  ON_TRACK: "On track",
  AT_RISK: "At risk",
  OFF_TRACK: "Off track",
};

export const ProjectUpdateNotification: React.FC<{
  notification: ProjectUpdateNotificationFragment;
}> = ({ notification: { projectUpdate } }) => (
  <>
    posted an update on <ProjectChip project={projectUpdate.project} />{" "}
    <span className={classes.health} data-health={projectUpdate.health}>
      {healthLabels[projectUpdate.health]}
    </span>
    <span className={classes.body}>{excerpt(projectUpdate.body)}</span>
  </>
);
