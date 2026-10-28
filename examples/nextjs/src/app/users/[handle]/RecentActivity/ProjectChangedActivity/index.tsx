import { ProjectChip } from "@/components/ProjectChip";
import { UserChip } from "@/components/UserChip";
import { ActivityIssueLink } from "../ActivityIssueLink";
import type { ProjectChangedActivityFragment } from "./ProjectChangedActivityFragment.graphql";

export const ProjectChangedActivity: React.FC<{
  event: ProjectChangedActivityFragment;
}> = ({ event }) => (
  <>
    <UserChip user={event.actor} />{" "}
    {event.toProject !== null ? (
      <>
        moved <ActivityIssueLink issue={event.issue} /> to{" "}
        <ProjectChip project={event.toProject} />
      </>
    ) : (
      <>
        removed <ActivityIssueLink issue={event.issue} /> from{" "}
        {event.fromProject !== null ? (
          <ProjectChip project={event.fromProject} />
        ) : (
          "its project"
        )}
      </>
    )}
  </>
);
