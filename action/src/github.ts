import type { Finding, Report } from "./types.ts";

export interface HttpInit {
  method?: string;
  headers?: Record<string, string>;
  body?: string;
}

export interface HttpResponse {
  ok: boolean;
  status: number;
  json(): Promise<unknown>;
}

/** The part of `fetch` the action needs, so tests can supply a fake. */
export type FetchLike = (url: string, init?: HttpInit) => Promise<HttpResponse>;

export class GitHubError extends Error {
  readonly status: number;

  constructor(message: string, status: number) {
    super(message);
    this.name = "GitHubError";
    this.status = status;
  }
}

export interface CommentTarget {
  /** For example https://api.github.com */
  apiUrl: string;
  /** owner/repo */
  repository: string;
  issueNumber: number;
  token: string;
}

interface Comment {
  id: number;
  body: string;
}

const COMMENTS_PER_PAGE = 100;
const MAX_PAGES = 10;

function authHeaders(token: string): Record<string, string> {
  return {
    accept: "application/vnd.github+json",
    authorization: `Bearer ${token}`,
    "content-type": "application/json",
    "x-github-api-version": "2022-11-28",
  };
}

function expectOk(response: HttpResponse, action: string): void {
  if (!response.ok) {
    throw new GitHubError(`GitHub refused to ${action} (status ${response.status})`, response.status);
  }
}

function toComments(raw: unknown): Comment[] {
  if (!Array.isArray(raw)) {
    throw new Error("GitHub returned an unexpected comment list");
  }
  const comments: Comment[] = [];
  for (const item of raw) {
    if (
      typeof item === "object" &&
      item !== null &&
      typeof item.id === "number" &&
      typeof item.body === "string"
    ) {
      comments.push({ id: item.id, body: item.body });
    }
  }
  return comments;
}

async function findComment(
  http: FetchLike,
  target: CommentTarget,
  marker: string,
): Promise<number | null> {
  const base = target.apiUrl.replace(/\/+$/, "");
  for (let page = 1; page <= MAX_PAGES; page += 1) {
    const url = `${base}/repos/${target.repository}/issues/${target.issueNumber}/comments?per_page=${COMMENTS_PER_PAGE}&page=${page}`;
    const response = await http(url, { headers: authHeaders(target.token) });
    expectOk(response, "list comments");
    const comments = toComments(await response.json());
    const found = comments.find((comment) => comment.body.startsWith(marker));
    if (found !== undefined) {
      return found.id;
    }
    if (comments.length < COMMENTS_PER_PAGE) {
      return null;
    }
  }
  return null;
}

/**
 * Updates the comment that starts with `marker`, or creates one.
 * When the existing comment cannot be edited, a new one is created instead.
 */
export async function upsertComment(
  http: FetchLike,
  target: CommentTarget,
  marker: string,
  body: string,
): Promise<"created" | "updated"> {
  const base = target.apiUrl.replace(/\/+$/, "");
  const headers = authHeaders(target.token);
  const existing = await findComment(http, target, marker);
  if (existing !== null) {
    const response = await http(`${base}/repos/${target.repository}/issues/comments/${existing}`, {
      method: "PATCH",
      headers,
      body: JSON.stringify({ body }),
    });
    if (response.ok) {
      return "updated";
    }
    if (response.status !== 403 && response.status !== 404) {
      expectOk(response, "update the comment");
    }
  }
  const created = await http(
    `${base}/repos/${target.repository}/issues/${target.issueNumber}/comments`,
    { method: "POST", headers, body: JSON.stringify({ body }) },
  );
  expectOk(created, "create the comment");
  return "created";
}

export interface UploadTarget {
  apiUrl: string;
  apiKey: string;
  project: string;
  gitRef: string | null;
}

function toWire(finding: Finding): Record<string, unknown> {
  return {
    rule_id: finding.ruleId,
    category: finding.category,
    severity: finding.severity,
    message: finding.message,
    file: finding.file,
    line: finding.line,
    fix: finding.fix,
  };
}

/** Stores the report in a Shipcheck API. The server computes its own score. */
export async function uploadReport(
  http: FetchLike,
  target: UploadTarget,
  report: Report,
): Promise<void> {
  const response = await http(`${target.apiUrl.replace(/\/+$/, "")}/api/v1/scans`, {
    method: "POST",
    headers: { "content-type": "application/json", "x-api-key": target.apiKey },
    body: JSON.stringify({
      project: target.project,
      git_ref: target.gitRef,
      findings: report.findings.map(toWire),
    }),
  });
  if (!response.ok) {
    throw new Error(`The Shipcheck API refused the report (status ${response.status})`);
  }
}
