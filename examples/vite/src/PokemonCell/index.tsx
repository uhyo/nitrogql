import type { PokemonCell as PokemonCellFragment } from "./PokemonCell.graphql";

export const PokemonCell: React.FC<{
  pokemon: PokemonCellFragment;
}> = ({ pokemon }) => {
  const ja = pokemon.names.find((name) => name.language_id === 1);
  const en = pokemon.names.find((name) => name.language_id === 9);
  return (
    <li key={pokemon.id}>
      #{pokemon.id} <b>{ja?.name}</b> {en?.name}
    </li>
  );
};
