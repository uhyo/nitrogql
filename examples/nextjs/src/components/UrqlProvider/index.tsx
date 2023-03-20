"use client";

import { useMemo } from "react";
import { Client, Provider } from "urql";

export const UrqlProvider: React.FC<React.PropsWithChildren> = ({
  children,
}) => {
  const client = useMemo(() => {
    return new Client({
      url: "/api/graphql",
    });
  }, []);
  return <Provider value={client}>{children}</Provider>;
};
