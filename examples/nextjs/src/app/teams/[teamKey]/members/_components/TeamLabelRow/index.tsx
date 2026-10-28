import Link from "next/link";
import { LabelBadge } from "@/components/LabelBadge";
import type { TeamLabelRowFragment } from "./TeamLabelRowFragment.graphql";

export const TeamLabelRow: React.FC<{
  teamKey: string;
  label: TeamLabelRowFragment;
}> = ({ teamKey, label }) => (
  <tr>
    <td>
      <LabelBadge label={label} />
    </td>
    <td className="muted">{label.description ?? "—"}</td>
    <td>
      <Link href={`/teams/${teamKey}/issues?label=${label.id}`}>
        {label.openIssueCount} open{" "}
        {label.openIssueCount === 1 ? "issue" : "issues"}
      </Link>
    </td>
  </tr>
);
