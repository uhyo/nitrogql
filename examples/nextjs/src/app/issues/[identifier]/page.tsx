"use client";

import { useParams } from "next/navigation";
import { useQuery } from "urql";
import { Reactions } from "@/components/Reactions";
import { IssueAttachments } from "./IssueAttachments";
import { IssueDescription } from "./IssueDescription";
import { IssueHeader } from "./IssueHeader";
import { IssueProperties } from "./IssueProperties";
import { IssueRelations } from "./IssueRelations";
import { IssueTimeline } from "./IssueTimeline";
import IssueDetailQuery from "./issueDetail.graphql";
import classes from "./page.module.css";

export default function IssueDetailPage() {
  const { identifier } = useParams<{ identifier: string }>();
  const [{ data, fetching, error }] = useQuery({
    query: IssueDetailQuery,
    variables: { identifier },
  });

  if (error) {
    return (
      <div className="page">
        <p className="muted">{error.message}</p>
      </div>
    );
  }
  if (!data) {
    return (
      <div className="page">
        {fetching ? <p className="muted">Loading…</p> : null}
      </div>
    );
  }
  const { issue } = data;
  if (issue === null) {
    return (
      <div className="page">
        <p className="muted">Issue {identifier} does not exist.</p>
      </div>
    );
  }
  return (
    <div className={classes.page}>
      <div className={classes.main}>
        <IssueHeader issue={issue} />
        <IssueDescription issue={issue} />
        <Reactions subjectId={issue.id} reactions={issue.reactions} />
        <IssueRelations issue={issue} />
        <IssueAttachments issue={issue} />
        <IssueTimeline issue={issue} />
      </div>
      <aside className={classes.aside}>
        <IssueProperties issue={issue} />
      </aside>
    </div>
  );
}
