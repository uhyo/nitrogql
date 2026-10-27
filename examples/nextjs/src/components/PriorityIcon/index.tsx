import type { IssuePriority } from "@/schema/base.graphql";

export const priorityLabels: Record<IssuePriority, string> = {
  NO_PRIORITY: "No priority",
  URGENT: "Urgent",
  HIGH: "High",
  MEDIUM: "Medium",
  LOW: "Low",
};

/** Ordered from the highest priority. */
export const priorities: readonly IssuePriority[] = [
  "URGENT",
  "HIGH",
  "MEDIUM",
  "LOW",
  "NO_PRIORITY",
];

export const PriorityIcon: React.FC<{
  priority: IssuePriority;
  showName?: boolean;
}> = ({ priority, showName = false }) => {
  const bars = { NO_PRIORITY: 0, LOW: 1, MEDIUM: 2, HIGH: 3, URGENT: 3 }[
    priority
  ];
  return (
    <span
      title={priorityLabels[priority]}
      style={{ display: "inline-flex", alignItems: "center", gap: "0.4em" }}
    >
      <svg width="14" height="14" viewBox="0 0 14 14" aria-hidden>
        {priority === "URGENT" ? (
          <>
            <rect x="1" y="1" width="12" height="12" rx="3" fill="#f2994a" />
            <path d="M7 3.5v4.5M7 9.8v0.7" stroke="white" strokeWidth="1.6" />
          </>
        ) : priority === "NO_PRIORITY" ? (
          <path
            d="M2 7h2M6 7h2M10 7h2"
            stroke="currentColor"
            strokeOpacity="0.5"
            strokeWidth="1.5"
          />
        ) : (
          [0, 1, 2].map((i) => (
            <rect
              key={i}
              x={1.5 + i * 4}
              y={9 - i * 3}
              width="3"
              height={4 + i * 3}
              rx="0.5"
              fill="currentColor"
              fillOpacity={i < bars ? 0.9 : 0.25}
            />
          ))
        )}
      </svg>
      {showName ? <span>{priorityLabels[priority]}</span> : null}
    </span>
  );
};
