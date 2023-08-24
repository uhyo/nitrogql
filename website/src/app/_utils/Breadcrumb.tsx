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
        <>
          <Link href={href}>{label}</Link> &gt;{" "}
        </>
      ))}
      {current}
    </nav>
  );
};
