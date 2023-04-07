import classes from "./Figures.module.css";

export const Figures: React.FC<React.PropsWithChildren> = ({ children }) => {
  return <div className={classes.figures}>{children}</div>;
};
