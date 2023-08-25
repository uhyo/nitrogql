import Link from "next/link";
import "./Footer.css";

export const Footer = () => {
  return (
    <footer>
      <nav>
        <h2>nitrogql</h2>
        <ul>
          <li>
            <Link href="/">Top Page</Link>
          </li>
          <li>
            <Link href="/guides">Guides</Link>
          </li>
          <li>
            <Link href="/configuration">Configuration</Link>
          </li>
          <li>
            <Link href="/cli">CLI</Link>
          </li>
          <li>
            <Link href="/recipes">Recipes</Link>
          </li>
          <li>
            <Link href="/faq">FAQ</Link>
          </li>
        </ul>
        <ul>
          <li>
            <a href="https://github.com/uhyo/nitrogql" target="_blank">
              GitHub
            </a>
          </li>
        </ul>
      </nav>
    </footer>
  );
};
