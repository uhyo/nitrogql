import { ProjectChip } from "@/components/ProjectChip";
import { IssueTimelineEvent } from "../IssueTimelineEvent";
import type { IssueTimelineProjectChangedFragment } from "./IssueTimelineProjectChangedFragment.graphql";

export const IssueTimelineProjectChanged: React.FC<{
  event: IssueTimelineProjectChangedFragment;
}> = ({ event }) => {
  const { fromProject, toProject } = event;
  return (
    <IssueTimelineEvent event={event} icon="▣">
      {toProject !== null ? (
        <>
          <span className="muted">
            {fromProject !== null
              ? "moved the issue from"
              : "added the issue to"}
          </span>
          {fromProject !== null ? (
            <>
              <ProjectChip project={fromProject} />
              <span className="muted">to</span>
            </>
          ) : null}
          <ProjectChip project={toProject} />
        </>
      ) : fromProject !== null ? (
        <>
          <span className="muted">removed the issue from</span>
          <ProjectChip project={fromProject} />
        </>
      ) : (
        <span className="muted">removed the project</span>
      )}
    </IssueTimelineEvent>
  );
};
