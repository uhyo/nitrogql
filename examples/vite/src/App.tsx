import { useState } from "react";
import "./App.css";
import { PokemonList } from "./PokemonList";

function App() {
  const [count, setCount] = useState(0);

  return (
    <div className="App">
      <h1>Vite + React with nitrogql example</h1>
      <h2>A list of pokemons</h2>
      <p>
        Powered by{" "}
        <a href="https://pokeapi.co/" target="_blank">
          PokéAPI
        </a>
      </p>
      <PokemonList />
    </div>
  );
}

export default App;
