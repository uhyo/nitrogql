import { Fragment } from "react";
import Link from "next/link";
import classes from "./Breadcrumb.module.css";

type BreadcrumbProps = {
  parents: readonly {
    label: string;
    href: string;
  }[];
  current: string;
};

export const Breadcrumb = ({ parents, current }: BreadcrumbProps) => {
  return (
    <nav className={classes.nav}>
      {parents.map(({ label, href }) => (
        <Fragment key={label}>
          <Link href={href}>{label}</Link> &gt;{" "}
        </Fragment>
      ))}
      {current}
    </nav>
  );
};
