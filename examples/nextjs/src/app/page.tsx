"use client";

import { Inter } from "next/font/google";
import { Suspense } from "react";
import styles from "./page.module.css";
import { TodoList } from "./TodoList";
import { useControls } from "./useControls";

const inter = Inter({ subsets: ["latin"] });

export default function Home() {
  const [filter, controls] = useControls();
  return (
    <main className={styles.main}>
      <h1>nitrogql + Next.js Example: TODO app just for you</h1>
      {controls}
      <Suspense fallback={null}>
        <TodoList filter={filter} />
      </Suspense>
    </main>
  );
}
