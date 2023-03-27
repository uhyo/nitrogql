import { useQuery } from "@apollo/client";
import { useState } from "react";
import Query from "./pokemonList.graphql";

const pageSize = 50;

export const PokemonList: React.FC = () => {
  const [page, setPage] = useState(0);
  const { data, error, loading } = useQuery(Query, {
    variables: {
      limit: pageSize,
      offset: page * pageSize,
    },
  });

  if (loading) {
    return <div>Loading...</div>;
  }

  if (error) {
    return <div>Error: {error.message}</div>;
  }

  return (
    <div>
      <ul>
        {data.map((pokemon) => (
          <li key={pokemon.id}>{pokemon.name}</li>
        ))}
      </ul>
      <button disabled={page === 0} onClick={() => setPage((page) => page - 1)}>
        Previous
      </button>
      <button
        disabled={data.pokemons.results.length < pageSize}
        onClick={() => setPage((page) => page + 1)}
      >
        Next
      </button>
    </div>
  );
};
