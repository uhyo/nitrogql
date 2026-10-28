"use client";

import { useState } from "react";
import { useMutation, useQuery } from "urql";
import { NotificationItem } from "./NotificationItem";
import InboxQuery from "./inbox.graphql";
import MarkAllNotificationsReadMutation from "./markAllNotificationsRead.graphql";
import classes from "./inbox.module.css";

const pageSize = 30;

export default function InboxPage() {
  const [unreadOnly, setUnreadOnly] = useState(false);
  const [first, setFirst] = useState(pageSize);
  const [{ data, fetching, error }] = useQuery({
    query: InboxQuery,
    variables: { unreadOnly, first },
  });
  const [markAllResult, markAllRead] = useMutation(
    MarkAllNotificationsReadMutation,
  );
  const notifications = data?.notifications;

  return (
    <div className="page">
      <header className="pageHeader">
        <h1>Inbox</h1>
        {notifications ? (
          <span className="muted">{notifications.unreadCount} unread</span>
        ) : null}
        <label className={classes.toggle}>
          <input
            type="checkbox"
            checked={unreadOnly}
            onChange={(e) => {
              setUnreadOnly(e.target.checked);
              setFirst(pageSize);
            }}
          />
          Unread only
        </label>
        <button
          type="button"
          className="button"
          disabled={
            markAllResult.fetching ||
            notifications === undefined ||
            notifications.unreadCount === 0
          }
          onClick={() =>
            // The result is a number, so name the typename to refetch explicitly.
            markAllRead({}, { additionalTypenames: ["NotificationConnection"] })
          }
        >
          Mark all as read
        </button>
      </header>
      {error ? <p className="muted">{error.message}</p> : null}
      {markAllResult.error ? (
        <p className="muted">{markAllResult.error.message}</p>
      ) : null}
      {notifications ? (
        notifications.nodes.length === 0 ? (
          <p className={`${classes.empty} muted`}>
            {unreadOnly ? "No unread notifications" : "No notifications"}
          </p>
        ) : (
          <div className={classes.list}>
            <ul>
              {notifications.nodes.map((notification) => (
                <NotificationItem
                  key={notification.id}
                  notification={notification}
                />
              ))}
            </ul>
            <footer className={`${classes.footer} muted`}>
              {notifications.nodes.length} of {notifications.totalCount}{" "}
              notifications
              {notifications.pageInfo.hasNextPage ? (
                <button
                  type="button"
                  className="button"
                  onClick={() => setFirst((first) => first + pageSize)}
                >
                  Load more
                </button>
              ) : null}
            </footer>
          </div>
        )
      ) : fetching ? (
        <p className="muted">Loading…</p>
      ) : null}
    </div>
  );
}
