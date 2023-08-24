import classes from "./InPageNav.module.css";

export const InPageNav = ({ children }: React.PropsWithChildren) => {
  return <nav className={classes.nav}>{children}</nav>;
};
