import { StateIcon } from "@/components/StateIcon";
import type { UpdateIssueInput } from "@/schema/base.graphql";
import type { IssueStateSelectFragment } from "./IssueStateSelectFragment.graphql";

/** Selects a workflow state of the issue's team. */
export const IssueStateSelect: React.FC<{
  issue: IssueStateSelectFragment;
  disabled?: boolean;
  onChange: (input: UpdateIssueInput) => void;
}> = ({ issue, disabled, onChange }) => {
  const states = [...issue.team.workflowStates].sort(
    (a, b) => a.position - b.position,
  );
  return (
    <>
      <StateIcon state={issue.state} />
      <select
        className="input"
        aria-label="State"
        value={issue.state.id}
        disabled={disabled}
        onChange={(e) => onChange({ stateId: e.target.value })}
      >
        {states.map((state) => (
          <option key={state.id} value={state.id}>
            {state.name}
          </option>
        ))}
      </select>
    </>
  );
};
