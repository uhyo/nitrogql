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
      <div className="toc">
        {children}
        <Outline toc={toc} currentHeading={currentHeading} />
      </div>
    </>
  );
};

function useCurrentHeading(toc: readonly TocItem[]): string | undefined {
  const [intersectingSet, setIntersectingSet] = useState<Set<string>>(
    new Set()
  );

  useEffect(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          const id = entry.target.id;
          if (id === undefined) {
            continue;
          }
          if (entry.intersectionRatio === 1) {
            setIntersectingSet((prev) => {
              const next = new Set(prev);
              next.add(id);
              return next;
            });
          } else {
            setIntersectingSet((prev) => {
              const next = new Set(prev);
              next.delete(id);
              return next;
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
    const firstInteresctionHeader = toc.findIndex(
      (item) => item.id !== undefined && intersectingSet.has(item.id)
    );
    if (firstInteresctionHeader === -1) {
      return undefined;
    }
    return toc.at(firstInteresctionHeader - 1)?.id;
  }, [toc, intersectingSet]);

  return currentHeader;
}
