import Image from "next/image";
import LogoImage from "../../public/nitrogql-logo.png";
import { Outfit } from "next/font/google";
import styles from "./page.module.css";

const font = Outfit({ subsets: ["latin"] });

export default function Home() {
  return (
    <>
      <div className={styles.main}>
        <hgroup className={font.className}>
          <p>
            <Image src={LogoImage} alt="nitrogql logo" />
          </p>
          <h1>
            <span>nitrogql</span>
          </h1>
          <p>
            GraphQL + TypeScript <wbr />
            Done Right.
          </p>
        </hgroup>
        <main>
          <p>
            <b>nitrogql</b> is a toolchain for using GraphQL with TypeScript. It
            can <string>generate TypeScript types</string> from your GraphQL
            schema and queries, and also{" "}
            <strong>provides static checking</strong> for your queries.
          </p>
        </main>
      </div>
    </>
  );
}
