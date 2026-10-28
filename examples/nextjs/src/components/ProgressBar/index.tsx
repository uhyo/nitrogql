import { formatPercent } from "@/lib/format";

export const ProgressBar: React.FC<{
  progress: number;
  color?: string;
  showLabel?: boolean;
}> = ({ progress, color = "var(--color-accent)", showLabel = true }) => (
  <span style={{ display: "inline-flex", alignItems: "center", gap: "0.5em" }}>
    <span
      role="progressbar"
      aria-valuenow={Math.round(progress * 100)}
      aria-valuemin={0}
      aria-valuemax={100}
      style={{
        display: "inline-block",
        width: "6em",
        height: "6px",
        borderRadius: "3px",
        background: "var(--color-bg-hover)",
        overflow: "hidden",
      }}
    >
      <span
        style={{
          display: "block",
          width: formatPercent(progress),
          height: "100%",
          background: color,
        }}
      />
    </span>
    {showLabel ? (
      <span className="muted">{formatPercent(progress)}</span>
    ) : null}
  </span>
);
