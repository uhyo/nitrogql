const relativeFormat = new Intl.RelativeTimeFormat("en", { numeric: "auto" });
const units: [Intl.RelativeTimeFormatUnit, number][] = [
  ["year", 365 * 24 * 60 * 60],
  ["month", 30 * 24 * 60 * 60],
  ["week", 7 * 24 * 60 * 60],
  ["day", 24 * 60 * 60],
  ["hour", 60 * 60],
  ["minute", 60],
];

/** Formats a date relative to now, such as "3 days ago". */
export function formatRelative(date: string | Date, now = new Date()): string {
  const seconds = (new Date(date).getTime() - now.getTime()) / 1000;
  for (const [unit, unitSeconds] of units) {
    if (Math.abs(seconds) >= unitSeconds) {
      return relativeFormat.format(Math.round(seconds / unitSeconds), unit);
    }
  }
  return "just now";
}

const dateFormat = new Intl.DateTimeFormat("en", {
  month: "short",
  day: "numeric",
});
const dateTimeFormat = new Intl.DateTimeFormat("en", {
  dateStyle: "medium",
  timeStyle: "short",
});

/** Formats a date, such as "May 1". `Date` values (YYYY-MM-DD) are treated as UTC. */
export function formatDate(date: string): string {
  return dateFormat.format(new Date(date));
}

/** Formats a date and time, such as "May 1, 2024, 9:00 AM". */
export function formatDateTime(date: string): string {
  return dateTimeFormat.format(new Date(date));
}

/** Formats a ratio as percentage, such as "42%". */
export function formatPercent(ratio: number): string {
  return `${Math.round(ratio * 100)}%`;
}
