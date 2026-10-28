import { IssueRow } from "@/components/IssueRow";
import type { SearchIssueResultFragment } from "./SearchIssueResultFragment.graphql";

export const SearchIssueResult: React.FC<{
  issue: SearchIssueResultFragment;
}> = ({ issue }) => <IssueRow issue={issue} />;
