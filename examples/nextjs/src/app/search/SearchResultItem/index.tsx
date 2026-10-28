import { SearchIssueResult } from "./SearchIssueResult";
import { SearchProjectResult } from "./SearchProjectResult";
import { SearchTeamResult } from "./SearchTeamResult";
import { SearchUserResult } from "./SearchUserResult";
import type { SearchResultItemFragment } from "./SearchResultItemFragment.graphql";

/** One result of global search. */
export const SearchResultItem: React.FC<{
  result: SearchResultItemFragment;
}> = ({ result }) => {
  switch (result.__typename) {
    case "Issue":
      return <SearchIssueResult issue={result} />;
    case "Project":
      return <SearchProjectResult project={result} />;
    case "User":
      return <SearchUserResult user={result} />;
    case "Team":
      return <SearchTeamResult team={result} />;
  }
};
