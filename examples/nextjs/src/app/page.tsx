import { Inter } from "next/font/google";
import { Suspense } from "react";
import styles from "./page.module.css";
import { TodoList } from "./TodoList";

const inter = Inter({ subsets: ["latin"] });

export default function Home() {
  return (
    <main className={styles.main}>
      <h1>nitrogql + Next.js Example: TODO app just for you</h1>
      <Suspense fallback={null}>
        <TodoList />
      </Suspense>
    </main>
  );
}
