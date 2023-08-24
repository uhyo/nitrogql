"use client";

import { useState, useMemo, useEffect } from "react";
import { TocItem, generateTocFromReactNode } from "./generateTocFromReactNode";
import { Outline } from "./Outline";
import "./Toc.css";

export const Toc: React.FC<React.PropsWithChildren> = ({ children }) => {
  const toc = useMemo(() => {
    const res = generateTocFromReactNode(children);
    return res.filter((item) => item.id !== undefined);
  }, [children]);

  const currentHeading = useCurrentHeading(toc);

  return (
    <>
      {children}
      <div className="toc">
        <Outline toc={toc} currentHeading={currentHeading} />
      </div>
    </>
  );
};

function useCurrentHeading(toc: readonly TocItem[]): string | undefined {
  const [{ intersectingSet, currentHeaderByPosition }, setIntersectingSet] =
    useState<{
      intersectingSet: Set<string>;
      currentHeaderByPosition: string | undefined;
    }>(() => ({
      intersectingSet: new Set(),
      currentHeaderByPosition: undefined,
    }));

  useEffect(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          const id = entry.target.id;
          if (id === undefined) {
            continue;
          }
          if (entry.intersectionRatio === 1) {
            setIntersectingSet(({ intersectingSet }) => {
              const next = new Set(intersectingSet);
              next.add(id);
              return {
                intersectingSet: next,
                currentHeaderByPosition: undefined,
              };
            });
          } else {
            setIntersectingSet(({ intersectingSet }) => {
              const next = new Set(intersectingSet);
              next.delete(id);
              if (next.size !== 0) {
                return {
                  intersectingSet: next,
                  currentHeaderByPosition: undefined,
                };
              }
              return {
                intersectingSet: next,
                currentHeaderByPosition: getCurrentHeaderByPosition(toc),
              };
            });
          }
        }
      },
      {
        threshold: 1,
        rootMargin: "-4px",
      }
    );

    for (const item of toc) {
      if (item.id === undefined) {
        continue;
      }
      const element = document.getElementById(item.id);
      if (element) {
        observer.observe(element);
      }
    }

    return () => {
      observer.disconnect();
    };
  }, [toc]);

  const currentHeader = useMemo(() => {
    if (intersectingSet.size === 0) {
      return currentHeaderByPosition;
    }
    const firstIntersectionHeader = toc.findIndex(
      (item) => item.id !== undefined && intersectingSet.has(item.id)
    );
    if (firstIntersectionHeader === -1) {
      return undefined;
    }
    return toc.at(
      firstIntersectionHeader === 0 ? 0 : firstIntersectionHeader - 1
    )?.id;
  }, [toc, intersectingSet, currentHeaderByPosition]);

  return currentHeader;
}

function getCurrentHeaderByPosition(
  toc: readonly TocItem[]
): string | undefined {
  let start = 0;
  let end = toc.length - 1;
  while (start <= end) {
    const mid = Math.floor((start + end) / 2);
    const item = toc[mid];
    if (item.id === undefined) {
      return undefined;
    }
    const element = document.getElementById(item.id);
    if (element === null) {
      return undefined;
    }
    const rect = element.getBoundingClientRect();
    if (rect.top < 0) {
      start = mid + 1;
    } else {
      end = mid - 1;
    }
  }
  if (start === 0) {
    return undefined;
  }
  return toc[start - 1].id;
}
