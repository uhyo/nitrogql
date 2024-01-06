"use client";

import { useMemo } from "react";
import { Provider, cacheExchange, createClient, fetchExchange } from "urql";

export const UrqlProvider: React.FC<React.PropsWithChildren> = ({
  children,
}) => {
  const client = useMemo(() => {
    return createClient({
      url: "/graphql",
      exchanges: [cacheExchange, fetchExchange],
    });
  }, []);
  return <Provider value={client}>{children}</Provider>;
};
