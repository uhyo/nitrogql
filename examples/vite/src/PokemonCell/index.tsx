import { useFragment } from "@apollo/client";
import { PokemonCellFragment } from "./PokemonCell.graphql";

export const PokemonCell: React.FC<{
  id: number;
}> = ({ id }) => {
  const { complete, data: pokemon } = useFragment({
    from: {
      __typename: "pokemon_v2_pokemonspecies",
      id,
    },
    fragment: PokemonCellFragment,
  });
  if (!complete) {
    throw new Error("Data is not complete");
  }
  const ja = pokemon.names.find((name) => name.language_id === 1);
  const en = pokemon.names.find((name) => name.language_id === 9);
  return (
    <li key={pokemon.id}>
      #{pokemon.id} <b>{ja?.name}</b> {en?.name}
    </li>
  );
};
