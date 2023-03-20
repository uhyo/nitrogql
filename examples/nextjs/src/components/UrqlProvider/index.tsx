"use client";

import { useMemo } from "react";
import { Provider, createClient } from "urql";

export const UrqlProvider: React.FC<React.PropsWithChildren> = ({
  children,
}) => {
  const client = useMemo(() => {
    return createClient({
      url: "/graphql",
    });
  }, []);
  return <Provider value={client}>{children}</Provider>;
};
