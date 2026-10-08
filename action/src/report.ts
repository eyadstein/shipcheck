import { CATEGORIES, SEVERITIES } from "./types.ts";
import type { Category, Finding, Report, Severity } from "./types.ts";

/** Hidden marker that lets the action find and update its own comment. */
export const MARKER = "<!-- shipcheck-report -->";
export const DEFAULT_MAX_FINDINGS = 25;
export const SUMMARY_MAX_FINDINGS = 200;
/** GitHub rejects comments longer than 65536 characters, so stay well below. */
const MAX_COMMENT_CHARS = 60_000;
const PROJECT_FILE = "(project)";

export interface RenderOptions {
  failUnder: number;
  /** Scanned directory relative to the repository root, "" for the root. */
  scanPath: string;
  repository: string | null;
  sha: string | null;
  serverUrl: string;
  maxFindings?: number;
}

/** Reads the `fail-under` input: a whole number from 0 to 100, empty means 0. */
export function parseThreshold(value: string): number {
  const trimmed = value.trim();
  if (trimmed === "") {
    return 0;
  }
  const number = Number(trimmed);
  if (!/^\d{1,3}$/.test(trimmed) || number > 100) {
    throw new Error(`fail-under must be a whole number from 0 to 100, got "${value}"`);
  }
  return number;
}

/** Reads a true or false input. */
export function parseFlag(value: string, fallback: boolean): boolean {
  const lowered = value.trim().toLowerCase();
  if (lowered === "") return fallback;
  if (lowered === "true") return true;
  if (lowered === "false") return false;
  throw new Error(`Expected true or false, got "${value}"`);
}

/** Turns the `path` input into a repository relative path without slashes at the ends. */
export function normalizeScanPath(path: string): string {
  const cleaned = path.replace(/\\/g, "/").replace(/^(\.\/)+/, "").replace(/\/+$/, "");
  return cleaned === "." ? "" : cleaned;
}

/** Wraps text in a markdown code span that survives backticks inside the text. */
export function code(text: string): string {
  const flat = text.replace(/[\r\n]+/g, " ");
  const longest = Math.max(0, ...(flat.match(/`+/g) ?? []).map((run) => run.length));
  const fence = "`".repeat(longest + 1);
  const pad = flat.startsWith("`") || flat.endsWith("`") ? " " : "";
  return `${fence}${pad}${flat}${pad}${fence}`;
}

/** Makes free text safe for a comment: one line, no mentions, no raw HTML tags. */
export function plain(text: string): string {
  return text
    .replace(/[\r\n]+/g, " ")
    .replace(/@/g, "@\u200b")
    .replace(/</g, "&lt;");
}

export function scoreTone(score: number): string {
  if (score >= 90) return "Good";
  if (score >= 70) return "Fair";
  if (score >= 40) return "Poor";
  return "Failing";
}

function severityRank(severity: Severity): number {
  return SEVERITIES.indexOf(severity);
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

function capitalize(word: string): string {
  return word.charAt(0).toUpperCase() + word.slice(1);
}

function severityCounts(findings: readonly Finding[]): Record<Severity, number> {
  const counts: Record<Severity, number> = { info: 0, low: 0, medium: 0, high: 0, critical: 0 };
  for (const finding of findings) {
    counts[finding.severity] += 1;
  }
  return counts;
}

function categoryCounts(findings: readonly Finding[]): Record<Category, number> {
  const counts: Record<Category, number> = { legal: 0, security: 0, design: 0 };
  for (const finding of findings) {
    counts[finding.category] += 1;
  }
  return counts;
}

function encodeSegment(segment: string): string {
  return encodeURIComponent(segment).replace(/[()]/g, (char) => (char === "(" ? "%28" : "%29"));
}

type LinkOptions = Pick<RenderOptions, "scanPath" | "repository" | "sha" | "serverUrl">;

/** A link to the file at the scanned commit, or null when no safe link can be built. */
export function fileLink(file: string, line: number, options: LinkOptions): string | null {
  if (options.repository === null || options.sha === null) return null;
  if (file === "" || file === PROJECT_FILE) return null;
  const base = options.scanPath;
  if (base.startsWith("/") || base.split("/").includes("..")) return null;
  const path = base === "" ? file : `${base}/${file}`;
  const encoded = path.split("/").map(encodeSegment).join("/");
  const anchor = line > 0 ? `#L${line}` : "";
  const server = options.serverUrl.replace(/\/+$/, "");
  return `${server}/${options.repository}/blob/${options.sha}/${encoded}${anchor}`;
}

function verdict(score: number, failUnder: number): string {
  if (failUnder <= 0) {
    return "No minimum score is required.";
  }
  return score >= failUnder
    ? `Passed: the score is at or above the required ${failUnder}.`
    : `Failed: the score is below the required ${failUnder}.`;
}

function countTable(findings: readonly Finding[]): string[] {
  const counts = categoryCounts(findings);
  return [
    "| Category | Findings |",
    "| --- | --- |",
    ...CATEGORIES.map((category) => `| ${capitalize(category)} | ${counts[category]} |`),
  ];
}

function severityLine(findings: readonly Finding[]): string {
  const counts = severityCounts(findings);
  const parts = [...SEVERITIES]
    .reverse()
    .filter((severity) => counts[severity] > 0)
    .map((severity) => `${counts[severity]} ${severity}`);
  const noun = findings.length === 1 ? "finding" : "findings";
  return `${findings.length} ${noun}: ${parts.join(", ")}.`;
}

function renderFinding(finding: Finding, options: RenderOptions): string {
  const label = finding.line > 0 ? `${finding.file}:${finding.line}` : finding.file;
  const link = fileLink(finding.file, finding.line, options);
  const where = link === null ? code(label) : `[${code(label)}](${link})`;
  const head = `- **${capitalize(finding.severity)}** ${code(finding.ruleId)} at ${where}: ${plain(finding.message)}`;
  return finding.fix === null ? head : `${head}\n  - Fix: ${plain(finding.fix)}`;
}

function build(
  report: Report,
  sorted: readonly Finding[],
  options: RenderOptions,
  limit: number,
): string {
  const lines = [
    MARKER,
    `## Shipcheck: ${report.score}/100 (${scoreTone(report.score)})`,
    "",
    verdict(report.score, options.failUnder),
    "",
  ];
  if (sorted.length === 0) {
    lines.push("No findings.");
    return `${lines.join("\n")}\n`;
  }
  lines.push(...countTable(sorted), "", severityLine(sorted), "", "### Findings", "");
  for (const finding of sorted.slice(0, limit)) {
    lines.push(renderFinding(finding, options));
  }
  const hidden = sorted.length - limit;
  if (hidden > 0) {
    const noun = hidden === 1 ? "finding" : "findings";
    lines.push("", `${hidden} more ${noun} not shown. Run Shipcheck locally for the full list.`);
  }
  return `${lines.join("\n")}\n`;
}

/** Builds the markdown for the pull request comment and the job summary. */
export function renderComment(report: Report, options: RenderOptions): string {
  const sorted = [...report.findings].sort(compareFindings);
  let limit = Math.max(1, options.maxFindings ?? DEFAULT_MAX_FINDINGS);
  let text = build(report, sorted, options, limit);
  while (text.length > MAX_COMMENT_CHARS && limit > 1) {
    limit = Math.floor(limit / 2);
    text = build(report, sorted, options, limit);
  }
  return text.length > MAX_COMMENT_CHARS ? text.slice(0, MAX_COMMENT_CHARS) : text;
}
