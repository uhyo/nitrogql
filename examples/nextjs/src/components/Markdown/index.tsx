import classes from "./Markdown.module.css";

/**
 * Renders a small subset of Markdown: headings, lists, paragraphs and mentions.
 */
export const Markdown: React.FC<{ source: string }> = ({ source }) => {
  const blocks = source.split(/\n{2,}/);
  return (
    <div className={classes.markdown}>
      {blocks.map((block, index) => {
        const heading = /^(#{1,3}) (.*)$/.exec(block);
        if (heading) {
          const Tag = `h${heading[1].length + 2}` as "h3" | "h4" | "h5";
          return <Tag key={index}>{inline(heading[2])}</Tag>;
        }
        const lines = block.split("\n");
        if (lines.every((line) => /^(- |\d+\. )/.test(line))) {
          const List = lines[0].startsWith("- ") ? "ul" : "ol";
          return (
            <List key={index}>
              {lines.map((line, i) => (
                <li key={i}>{inline(line.replace(/^(- |\d+\. )/, ""))}</li>
              ))}
            </List>
          );
        }
        return <p key={index}>{inline(block)}</p>;
      })}
    </div>
  );
};

function inline(text: string): React.ReactNode[] {
  return text.split(/(@[a-z0-9_]+)/).map((part, index) =>
    part.startsWith("@") ? (
      <a
        key={index}
        href={`/users/${part.slice(1)}`}
        className={classes.mention}
      >
        {part}
      </a>
    ) : (
      part
    ),
  );
}
