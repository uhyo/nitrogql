import type { NotificationRecord } from "./records";

/** Model of connection types. */
export type Connection<T> = {
  nodes: T[];
  pageInfo: {
    hasNextPage: boolean;
    endCursor: string | null;
  };
  totalCount: number;
};

export type NotificationConnection = Connection<NotificationRecord> & {
  unreadCount: number;
};
