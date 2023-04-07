import hl from "highlight.js/lib/common";
import "highlight.js/styles/a11y-dark.css";

/**
 * Highlights given code.
 */
export const Highlight: React.FC<{ children: string; language: string }> = ({
  language,
  children,
}) => {
  const highlighted = hl.highlight(children, {
    language,
  });
  return (
    <pre>
      <code
        className={`hljs language-${highlighted.language}`}
        dangerouslySetInnerHTML={{ __html: highlighted.value }}
      />
    </pre>
  );
};
