import hl from "highlight.js/lib/common";
import "highlight.js/styles/a11y-dark.css";

/**
 * Highlights given code.
 */
export const Highlight: React.FC<{ children: string }> = ({ children }) => {
  const highlighted = hl.highlightAuto(children);
  return (
    <pre>
      <code
        className="hljs"
        dangerouslySetInnerHTML={{ __html: highlighted.value }}
      />
    </pre>
  );
};
