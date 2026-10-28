import { UserChip } from "@/components/UserChip";
import type { ProjectMembersFragment } from "./ProjectMembersFragment.graphql";
import classes from "./ProjectMembers.module.css";

export const ProjectMembers: React.FC<{ project: ProjectMembersFragment }> = ({
  project,
}) => {
  if (project.members.length === 0) {
    return <p className="muted">No members</p>;
  }
  return (
    <ul className={classes.members}>
      {project.members.map((member) => (
        <li key={member.id}>
          <UserChip user={member} showStatus />
          {member.id === project.lead?.id ? (
            <span className="muted">Lead</span>
          ) : null}
        </li>
      ))}
    </ul>
  );
};
