import type { Resolvers } from "@/generated/resolvers";
import { type Context, resolveTypeById } from "./context";
import * as events from "./events";
import { Mutation } from "./mutation";
import * as objects from "./objects";
import { Query } from "./query";

export type { Context };

export const resolvers: Resolvers<Context> = {
  Query,
  Mutation,
  ...objects,
  ...events,
  Node: { __resolveType: resolveTypeById() },
  IssueEvent: { __resolveType: resolveTypeById() },
  Notification: { __resolveType: resolveTypeById() },
  TimelineItem: { __resolveType: resolveTypeById() },
  SearchResult: { __resolveType: resolveTypeById() },
  ReactionTarget: { __resolveType: resolveTypeById() },
};
