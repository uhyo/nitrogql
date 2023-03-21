"use client";

import { useState } from "react";
import styles from "./page.module.css";

export type Filter = "all" | "unfinished";

export const useControls = (): [filter: Filter, controls: JSX.Element] => {
  const [filter, setFilter] = useState<Filter>("unfinished");

  const controls = (
    <div className={styles.controls}>
      <label>
        <input
          type="radio"
          checked={filter === "unfinished"}
          onChange={(e) => {
            if (e.target.checked) {
              setFilter("unfinished");
            }
          }}
        />
        Unfinished TODOs
      </label>
      <label>
        <input
          type="radio"
          checked={filter === "all"}
          onChange={(e) => {
            if (e.target.checked) {
              setFilter("all");
            }
          }}
        />
        All TODOs
      </label>
    </div>
  );

  return [filter, controls];
};
