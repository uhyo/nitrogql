import { Metadata } from "next";

/**
 * Default description
 */
const defaultDescription =
  "Documentation of nitrogql, a GraphQL + TypeScript toolchain";

type OGPInput = {
  title: string;
  description?: string;
};

/**
 * Generates metadata that includes OGPs.
 * To be used by individual pages.
 */
export function ogp({
  title,
  description = defaultDescription,
}: OGPInput): Metadata {
  return {
    title,
    description,
  };
}
