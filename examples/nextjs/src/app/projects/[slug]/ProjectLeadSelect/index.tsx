"use client";

import { useMutation } from "urql";
import type { ProjectLeadSelectFragment } from "./ProjectLeadSelectFragment.graphql";
import UpdateProjectLeadMutation from "./updateProjectLead.graphql";

/** Select for the lead of a project. Candidates are members of the project's team. */
export const ProjectLeadSelect: React.FC<{
  project: ProjectLeadSelectFragment;
}> = ({ project }) => {
  const [{ fetching, error }, updateLead] = useMutation(
    UpdateProjectLeadMutation,
  );
  const candidates = project.team.members
    .filter(({ role, user }) => role !== "GUEST" && !user.deactivated)
    .map(({ user }) => user)
    .sort((a, b) => a.name.localeCompare(b.name));
  return (
    <>
      <select
        className="input"
        aria-label="Lead"
        value={project.lead?.id ?? ""}
        disabled={fetching}
        onChange={(e) =>
          updateLead({ id: project.id, leadId: e.target.value || null })
        }
      >
        <option value="">No lead</option>
        {candidates.map((user) => (
          <option key={user.id} value={user.id}>
            {user.name} (@{user.handle})
          </option>
        ))}
      </select>
      {error ? <span className="muted">{error.message}</span> : null}
    </>
  );
};
