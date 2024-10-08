import Image from "next/image";
import Link from "next/link";
import Logo from "../../../public/nitrogql-logo.png";
import styles from "./Header.module.css";

export const Header: React.FC = () => {
  return (
    <>
      <header className={styles.header}>
        <Link href="/">
          <h1>
            <Image className={styles.logo} src={Logo} alt="nitrogql logo" />
            <span>nitrogql</span>
          </h1>
        </Link>
      </header>
      <HeadNav />
    </>
  );
};

export const HeadNav: React.FC = () => {
  return (
    <nav className={styles.nav}>
      <ul>
        <li>
          <Link href="/guides">Guides</Link>
        </li>
        <li>
          <Link href="/configuration">Configuration</Link>
        </li>
        <li>
          <Link href="/references">References</Link>
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
        <li>
          <Link href="/blog">Blog</Link>
        </li>
      </ul>
    </nav>
  );
};
