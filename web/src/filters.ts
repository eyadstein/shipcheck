import { CATEGORIES, SEVERITIES, type Category, type Finding, type Severity } from "./types";
import { severityRank } from "./format";

export interface FindingFilter {
  category: Category | "all";
  severity: Severity | "all";
  query: string;
}

export const NO_FILTER: FindingFilter = { category: "all", severity: "all", query: "" };

export function isFiltered(filter: FindingFilter): boolean {
  return filter.category !== "all" || filter.severity !== "all" || filter.query.trim() !== "";
}

export function parseCategory(value: string): Category | "all" {
  return CATEGORIES.find((category) => category === value) ?? "all";
}

export function parseSeverity(value: string): Severity | "all" {
  return SEVERITIES.find((severity) => severity === value) ?? "all";
}

/** Most severe first, then by file and line. */
export function compareFindings(a: Finding, b: Finding): number {
  return (
    severityRank(b.severity) - severityRank(a.severity) ||
    a.file.localeCompare(b.file) ||
    a.line - b.line ||
    a.ruleId.localeCompare(b.ruleId)
  );
}

function matchesQuery(finding: Finding, query: string): boolean {
  return [finding.ruleId, finding.message, finding.file].some((field) =>
    field.toLowerCase().includes(query),
  );
}

export function applyFilter(findings: readonly Finding[], filter: FindingFilter): Finding[] {
  const query = filter.query.trim().toLowerCase();
  return findings
    .filter((finding) => filter.category === "all" || finding.category === filter.category)
    .filter((finding) => filter.severity === "all" || finding.severity === filter.severity)
    .filter((finding) => query === "" || matchesQuery(finding, query))
    .sort(compareFindings);
}

export function severityCounts(findings: readonly Finding[]): Record<Severity, number> {
  const counts: Record<Severity, number> = { info: 0, low: 0, medium: 0, high: 0, critical: 0 };
  for (const finding of findings) {
    counts[finding.severity] += 1;
  }
  return counts;
}

export function categoryCounts(findings: readonly Finding[]): Record<Category, number> {
  const counts: Record<Category, number> = { legal: 0, security: 0, design: 0 };
  for (const finding of findings) {
    counts[finding.category] += 1;
  }
  return counts;
}
