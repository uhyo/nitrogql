import type { LabelBadgeFragment } from "./LabelBadgeFragment.graphql";
import classes from "./LabelBadge.module.css";

export const LabelBadge: React.FC<{
  label: LabelBadgeFragment;
  onRemove?: () => void;
}> = ({ label, onRemove }) => (
  <span className={classes.badge}>
    <span className={classes.dot} style={{ background: label.color }} />
    {label.name}
    {onRemove ? (
      <button
        type="button"
        className={classes.remove}
        aria-label={`Remove label ${label.name}`}
        onClick={onRemove}
      >
        ×
      </button>
    ) : null}
  </span>
);
