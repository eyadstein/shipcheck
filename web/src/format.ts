import { SEVERITIES, type Finding, type Severity } from "./types";

export type Tone = "good" | "fair" | "poor" | "bad";

export const TONE_LABEL: Record<Tone, string> = {
  good: "Good",
  fair: "Fair",
  poor: "Poor",
  bad: "Failing",
};

export function scoreTone(score: number): Tone {
  if (score >= 90) return "good";
  if (score >= 70) return "fair";
  if (score >= 40) return "poor";
  return "bad";
}

/** Higher means more severe. */
export function severityRank(severity: Severity): number {
  return SEVERITIES.indexOf(severity);
}

export function capitalize(word: string): string {
  return word.charAt(0).toUpperCase() + word.slice(1);
}

export function formatLocation(finding: Pick<Finding, "file" | "line">): string {
  return finding.line > 0 ? `${finding.file}:${finding.line}` : finding.file;
}

export function formatDate(iso: string): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) {
    return iso;
  }
  return date.toLocaleString("en-GB", { dateStyle: "medium", timeStyle: "short" });
}

function plural(count: number, unit: string): string {
  return `${count} ${unit}${count === 1 ? "" : "s"} ago`;
}

export function timeAgo(iso: string, now: number = Date.now()): string {
  const then = new Date(iso).getTime();
  if (Number.isNaN(then)) {
    return iso;
  }
  const seconds = Math.max(0, Math.round((now - then) / 1000));
  if (seconds < 60) return "just now";
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return plural(minutes, "minute");
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return plural(hours, "hour");
  const days = Math.floor(hours / 24);
  if (days < 30) return plural(days, "day");
  const months = Math.floor(days / 30);
  if (months < 12) return plural(months, "month");
  return plural(Math.max(1, Math.floor(days / 365)), "year");
}
