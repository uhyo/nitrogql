"use client";

import { usePathname, useRouter, useSearchParams } from "next/navigation";
import { Suspense, useState } from "react";
import { useQuery } from "urql";
import type { SearchKind } from "@/schema/base.graphql";
import { SearchResultItem } from "./SearchResultItem";
import type { SearchResultItemFragment } from "./SearchResultItem/SearchResultItemFragment.graphql";
import SearchPageQuery from "./search.graphql";
import classes from "./search.module.css";

const kinds: { value: SearchKind; label: string }[] = [
  { value: "ISSUE", label: "Issues" },
  { value: "PROJECT", label: "Projects" },
  { value: "USER", label: "People" },
  { value: "TEAM", label: "Teams" },
];

const typenameOfKind = {
  ISSUE: "Issue",
  PROJECT: "Project",
  USER: "User",
  TEAM: "Team",
} as const satisfies Record<SearchKind, SearchResultItemFragment["__typename"]>;

const pageSize = 30;

export default function SearchPage() {
  // useSearchParams() requires a Suspense boundary for static rendering.
  return (
    <Suspense>
      <SearchPageContent />
    </Suspense>
  );
}

const SearchPageContent: React.FC = () => {
  const router = useRouter();
  const pathname = usePathname();
  const searchParams = useSearchParams();
  const query = searchParams.get("q")?.trim() ?? "";
  const selectedKinds = parseKinds(searchParams.getAll("kind"));
  const [first, setFirst] = useState(pageSize);
  const [{ data, fetching, error }] = useQuery({
    query: SearchPageQuery,
    variables: {
      query,
      kinds: selectedKinds.length > 0 ? selectedKinds : null,
      first,
    },
    pause: query === "",
  });

  const navigate = (q: string, kinds: readonly SearchKind[]) => {
    const params = new URLSearchParams({ q });
    for (const kind of kinds) {
      params.append("kind", kind);
    }
    setFirst(pageSize);
    router.replace(`${pathname}?${params}`);
  };

  return (
    <div className="page">
      <header className="pageHeader">
        <h1>Search</h1>
      </header>
      <form
        key={query}
        className={classes.form}
        onSubmit={(e) => {
          e.preventDefault();
          const q = new FormData(e.currentTarget).get("q");
          navigate(typeof q === "string" ? q.trim() : "", selectedKinds);
        }}
      >
        <input
          type="search"
          name="q"
          className={`input ${classes.input}`}
          defaultValue={query}
          placeholder="Search issues, projects, people and teams…"
          aria-label="Search"
          autoFocus
        />
        <fieldset className={classes.kinds}>
          <legend className="muted">Show</legend>
          {kinds.map((kind) => (
            <label key={kind.value}>
              <input
                type="checkbox"
                checked={selectedKinds.includes(kind.value)}
                onChange={(e) =>
                  navigate(
                    query,
                    e.target.checked
                      ? [...selectedKinds, kind.value]
                      : selectedKinds.filter((k) => k !== kind.value),
                  )
                }
              />
              {kind.label}
            </label>
          ))}
        </fieldset>
      </form>
      {query === "" ? (
        <p className="muted">Type something to search.</p>
      ) : error ? (
        <p className="muted">{error.message}</p>
      ) : data ? (
        <SearchResults
          results={data.search}
          kinds={selectedKinds}
          onLoadMore={
            data.search.length >= first
              ? () => setFirst((first) => first + pageSize)
              : undefined
          }
        />
      ) : fetching ? (
        <p className="muted">Searching…</p>
      ) : null}
    </div>
  );
};

/** Search results grouped by their kinds. */
const SearchResults: React.FC<{
  results: readonly SearchResultItemFragment[];
  kinds: readonly SearchKind[];
  onLoadMore?: () => void;
}> = ({ results, kinds: selectedKinds, onLoadMore }) => {
  if (results.length === 0) {
    return <p className={`${classes.empty} muted`}>No results</p>;
  }
  const groups = kinds
    .filter(
      (kind) =>
        selectedKinds.length === 0 || selectedKinds.includes(kind.value),
    )
    .map((kind) => ({
      ...kind,
      results: results.filter(
        (result) => result.__typename === typenameOfKind[kind.value],
      ),
    }))
    .filter((group) => group.results.length > 0);
  return (
    <>
      {groups.map((group) => (
        <section key={group.value} className="section">
          <h2>
            {group.label} <span className="muted">{group.results.length}</span>
          </h2>
          <div className={classes.list}>
            {group.results.map((result) => (
              <SearchResultItem key={result.id} result={result} />
            ))}
          </div>
        </section>
      ))}
      {onLoadMore ? (
        <p className="section">
          <button type="button" className="button" onClick={onLoadMore}>
            Show more results
          </button>
        </p>
      ) : null}
    </>
  );
};

function parseKinds(values: readonly string[]): SearchKind[] {
  return kinds
    .map((kind) => kind.value)
    .filter((kind) => values.includes(kind));
}
