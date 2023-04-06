import styles from "./Hint.module.css";

export const Hint: React.FC = ({ children }) => {
  return <p className={styles.hint}>{children}</p>;
};
