import type { ProjectSummary, ScanSummary } from "./types";

/** Projects that need attention come first. */
export function sortProjects(projects: readonly ProjectSummary[]): ProjectSummary[] {
  return [...projects].sort(
    (a, b) => a.latestScore - b.latestScore || a.project.localeCompare(b.project),
  );
}

export function oldestFirst(scans: readonly ScanSummary[]): ScanSummary[] {
  return [...scans].sort((a, b) => Date.parse(a.createdAt) - Date.parse(b.createdAt));
}
