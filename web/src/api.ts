import {
  CATEGORIES,
  SEVERITIES,
  type Finding,
  type ProjectSummary,
  type ScanDetail,
  type ScanSummary,
} from "./types";

/** Base URL of the API. Empty means the same origin, which the dev proxy provides. */
const BASE: string = String(import.meta.env.VITE_API_URL ?? "").replace(/\/+$/, "");

export class ApiError extends Error {
  readonly status: number | null;

  constructor(message: string, status: number | null = null) {
    super(message);
    this.name = "ApiError";
    this.status = status;
  }
}

type Json = Record<string, unknown>;

function unexpected(what: string): ApiError {
  return new ApiError(`Unexpected response from the API: ${what}`);
}

function record(value: unknown, what: string): Json {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw unexpected(`${what} is not an object`);
  }
  return value as Json;
}

function list(value: unknown, what: string): unknown[] {
  if (!Array.isArray(value)) {
    throw unexpected(`${what} is not a list`);
  }
  return value;
}

function text(obj: Json, key: string): string {
  const value = obj[key];
  if (typeof value !== "string") {
    throw unexpected(`${key} is not text`);
  }
  return value;
}

function optionalText(obj: Json, key: string): string | null {
  const value = obj[key];
  if (value === null || value === undefined) {
    return null;
  }
  if (typeof value !== "string") {
    throw unexpected(`${key} is not text`);
  }
  return value;
}

function integer(obj: Json, key: string): number {
  const value = obj[key];
  if (typeof value !== "number" || !Number.isInteger(value)) {
    throw unexpected(`${key} is not a whole number`);
  }
  return value;
}

function oneOf<T extends string>(obj: Json, key: string, allowed: readonly T[]): T {
  const value = text(obj, key);
  const found = allowed.find((item) => item === value);
  if (found === undefined) {
    throw unexpected(`${key} has an unknown value`);
  }
  return found;
}

export function parseFinding(raw: unknown): Finding {
  const obj = record(raw, "finding");
  return {
    ruleId: text(obj, "rule_id"),
    category: oneOf(obj, "category", CATEGORIES),
    severity: oneOf(obj, "severity", SEVERITIES),
    message: text(obj, "message"),
    file: text(obj, "file"),
    line: integer(obj, "line"),
    fix: optionalText(obj, "fix"),
  };
}

export function parseScanSummary(raw: unknown): ScanSummary {
  const obj = record(raw, "scan");
  return {
    id: text(obj, "id"),
    project: text(obj, "project"),
    gitRef: optionalText(obj, "git_ref"),
    score: integer(obj, "score"),
    createdAt: text(obj, "created_at"),
    findingCount: integer(obj, "finding_count"),
  };
}

export function parseScanList(raw: unknown): ScanSummary[] {
  return list(raw, "scan list").map((item) => parseScanSummary(item));
}

export function parseScanDetail(raw: unknown): ScanDetail {
  const obj = record(raw, "scan");
  return {
    ...parseScanSummary(obj),
    findings: list(obj["findings"], "findings").map((item) => parseFinding(item)),
  };
}

export function parseProjectList(raw: unknown): ProjectSummary[] {
  return list(raw, "project list").map((item) => {
    const obj = record(item, "project");
    return {
      project: text(obj, "project"),
      latestScore: integer(obj, "latest_score"),
      lastScanAt: text(obj, "last_scan_at"),
      scanCount: integer(obj, "scan_count"),
    };
  });
}

export function scansPath(project: string | null, limit: number): string {
  const params = new URLSearchParams({ limit: String(limit) });
  if (project !== null) {
    params.set("project", project);
  }
  return `/api/v1/scans?${params.toString()}`;
}

async function errorMessage(response: Response): Promise<string> {
  try {
    const body: unknown = await response.json();
    if (typeof body === "object" && body !== null && "error" in body && typeof body.error === "string") {
      return body.error;
    }
  } catch {
    // The body was not JSON, so fall back to the generic message below.
  }
  return `The API answered with status ${response.status}`;
}

async function getJson(path: string, signal?: AbortSignal): Promise<unknown> {
  const init: RequestInit = { headers: { accept: "application/json" } };
  if (signal !== undefined) {
    init.signal = signal;
  }
  let response: Response;
  try {
    response = await fetch(`${BASE}${path}`, init);
  } catch (error) {
    if (signal?.aborted === true) {
      throw error;
    }
    throw new ApiError("Cannot reach the Shipcheck API. Is it running?");
  }
  if (!response.ok) {
    throw new ApiError(await errorMessage(response), response.status);
  }
  try {
    return await response.json();
  } catch {
    throw new ApiError("The API returned something that is not JSON", response.status);
  }
}

export async function fetchProjects(signal?: AbortSignal): Promise<ProjectSummary[]> {
  return parseProjectList(await getJson("/api/v1/projects", signal));
}

export async function fetchScans(
  project: string | null,
  limit: number,
  signal?: AbortSignal,
): Promise<ScanSummary[]> {
  return parseScanList(await getJson(scansPath(project, limit), signal));
}

export async function fetchScan(id: string, signal?: AbortSignal): Promise<ScanDetail> {
  return parseScanDetail(await getJson(`/api/v1/scans/${encodeURIComponent(id)}`, signal));
}
