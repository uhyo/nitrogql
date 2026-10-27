import { IssueRow } from "@/components/IssueRow";
import type { IssueRelationType } from "@/schema/base.graphql";
import type { IssueRelationsFragment } from "./IssueRelationsFragment.graphql";
import classes from "./IssueRelations.module.css";

const relationTypes: readonly IssueRelationType[] = [
  "BLOCKED_BY",
  "BLOCKS",
  "DUPLICATE_OF",
  "RELATED",
];

const relationTypeLabels: Record<IssueRelationType, string> = {
  BLOCKS: "Blocks",
  BLOCKED_BY: "Blocked by",
  RELATED: "Related to",
  DUPLICATE_OF: "Duplicate of",
};

/** Parent issue, sub-issues and related issues. */
export const IssueRelations: React.FC<{ issue: IssueRelationsFragment }> = ({
  issue,
}) => {
  const groups = relationTypes
    .map((type) => ({
      type,
      issues: issue.relations
        .filter((relation) => relation.type === type)
        .map((relation) => relation.relatedIssue),
    }))
    .filter((group) => group.issues.length > 0);
  if (
    issue.parent === null &&
    issue.children.length === 0 &&
    groups.length === 0
  ) {
    return null;
  }
  return (
    <>
      {issue.parent !== null ? (
        <section className="section">
          <h2>Parent issue</h2>
          <div className={classes.list}>
            <IssueRow issue={issue.parent} />
          </div>
        </section>
      ) : null}
      {issue.children.length > 0 ? (
        <section className="section">
          <h2>Sub-issues ({issue.children.length})</h2>
          <div className={classes.list}>
            {issue.children.map((child) => (
              <IssueRow key={child.id} issue={child} />
            ))}
          </div>
        </section>
      ) : null}
      {groups.length > 0 ? (
        <section className="section">
          <h2>Relations</h2>
          {groups.map((group) => (
            <div key={group.type} className={classes.group}>
              <h3 className={classes.groupTitle}>
                {relationTypeLabels[group.type]}
              </h3>
              <div className={classes.list}>
                {group.issues.map((relatedIssue) => (
                  <IssueRow key={relatedIssue.id} issue={relatedIssue} />
                ))}
              </div>
            </div>
          ))}
        </section>
      ) : null}
    </>
  );
};
