import { Avatar } from "@/components/Avatar";
import type { UpdateIssueInput } from "@/schema/base.graphql";
import type { IssueAssigneeSelectFragment } from "./IssueAssigneeSelectFragment.graphql";

const unassigned = "";

/** Selects a member of the issue's team as the assignee. */
export const IssueAssigneeSelect: React.FC<{
  issue: IssueAssigneeSelectFragment;
  disabled?: boolean;
  onChange: (input: UpdateIssueInput) => void;
}> = ({ issue, disabled, onChange }) => {
  const { assignee } = issue;
  // Deactivated users cannot be assigned, but the current assignee is kept
  const users = issue.team.members
    .map((member) => member.user)
    .filter((user) => !user.deactivated || user.id === assignee?.id);
  return (
    <>
      {assignee !== null ? <Avatar user={assignee} /> : null}
      <select
        className="input"
        aria-label="Assignee"
        value={assignee?.id ?? unassigned}
        disabled={disabled}
        onChange={(e) =>
          onChange({
            assigneeId: e.target.value === unassigned ? null : e.target.value,
          })
        }
      >
        <option value={unassigned}>Unassigned</option>
        {users.map((user) => (
          <option key={user.id} value={user.id}>
            {user.name} (@{user.handle})
          </option>
        ))}
      </select>
    </>
  );
};
