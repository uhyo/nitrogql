import { formatDateTime, formatRelative } from "@/lib/format";

/** Shows a date relative to now, with the exact date in a tooltip. */
export const RelativeTime: React.FC<{ date: string; className?: string }> = ({
  date,
  className,
}) => (
  <time dateTime={date} title={formatDateTime(date)} className={className}>
    {formatRelative(date)}
  </time>
);
