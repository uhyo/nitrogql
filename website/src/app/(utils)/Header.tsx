import Image from "next/image";
import Link from "next/link";
import Logo from "../../../public/nitrogql-logo.png";
import styles from "./Header.module.css";

export const Header: React.FC = () => {
  return (
    <header className={styles.header}>
      <Link href="/">
        <h1>
          <Image className={styles.logo} src={Logo} alt="nitrogql logo" />
          nitrogql
        </h1>
      </Link>
    </header>
  );
};
