import { TocItem } from "./generateTocFromReactNode";

export const Outline: React.FC<{
  toc: readonly TocItem[];
  currentHeading: string | undefined;
}> = ({ toc, currentHeading }) => {
  return (
    <nav>
      <h2>On this page</h2>
      <ul>
        {toc.map((item) => (
          <li
            key={item.id}
            data-level={item.level}
            data-current={item.id === currentHeading ? "true" : undefined}
          >
            <a href={`#${item.id}`}>{item.text}</a>
          </li>
        ))}
      </ul>
    </nav>
  );
};
