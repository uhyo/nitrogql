"use client";

import { useMutation } from "urql";
import { Avatar } from "@/components/Avatar";
import type { IssueSubscriptionFragment } from "./IssueSubscriptionFragment.graphql";
import SetIssueSubscriptionMutation from "./setIssueSubscription.graphql";
import classes from "./IssueSubscription.module.css";

const maxAvatars = 5;

/** Subscribers of an issue and a button to (un)subscribe. */
export const IssueSubscription: React.FC<{
  issue: IssueSubscriptionFragment;
}> = ({ issue }) => {
  const [{ fetching }, setSubscription] = useMutation(
    SetIssueSubscriptionMutation,
  );
  const { subscribers } = issue;
  return (
    <span className={classes.subscription}>
      <span
        className={classes.subscribers}
        title={`Subscribers: ${subscribers.map((user) => user.name).join(", ")}`}
      >
        {subscribers.slice(0, maxAvatars).map((user) => (
          <Avatar key={user.id} user={user} />
        ))}
        {subscribers.length > maxAvatars ? (
          <span className="muted">+{subscribers.length - maxAvatars}</span>
        ) : null}
      </span>
      <button
        type="button"
        className="button"
        disabled={fetching}
        aria-pressed={issue.viewerIsSubscribed}
        onClick={() =>
          setSubscription({
            issueId: issue.id,
            subscribed: !issue.viewerIsSubscribed,
          })
        }
      >
        {issue.viewerIsSubscribed ? "🔕 Unsubscribe" : "🔔 Subscribe"}
      </button>
    </span>
  );
};
