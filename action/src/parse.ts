import { CATEGORIES, SEVERITIES } from "./types.ts";
import type { Finding, Report } from "./types.ts";

type Json = Record<string, unknown>;

function fail(what: string): never {
  throw new Error(`Invalid Shipcheck report: ${what}`);
}

function record(value: unknown, what: string): Json {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    fail(`${what} is not an object`);
  }
  return value as Json;
}

function text(obj: Json, key: string): string {
  const value = obj[key];
  if (typeof value !== "string") {
    fail(`${key} is not text`);
  }
  return value;
}

function optionalText(obj: Json, key: string): string | null {
  const value = obj[key];
  if (value === null || value === undefined) {
    return null;
  }
  if (typeof value !== "string") {
    fail(`${key} is not text`);
  }
  return value;
}

function integer(obj: Json, key: string): number {
  const value = obj[key];
  if (typeof value !== "number" || !Number.isInteger(value)) {
    fail(`${key} is not a whole number`);
  }
  return value;
}

function oneOf<T extends string>(obj: Json, key: string, allowed: readonly T[]): T {
  const value = text(obj, key);
  const found = allowed.find((item) => item === value);
  if (found === undefined) {
    fail(`${key} has an unknown value`);
  }
  return found;
}

function parseFinding(raw: unknown): Finding {
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

/** Reads the JSON that `shipcheck --format json` prints and checks every field. */
export function parseReport(source: string): Report {
  let raw: unknown;
  try {
    raw = JSON.parse(source.replace(/^\uFEFF/, ""));
  } catch {
    fail("the file is not valid JSON");
  }
  const obj = record(raw, "report");
  const score = integer(obj, "score");
  if (score < 0 || score > 100) {
    fail("score is outside 0 to 100");
  }
  const findings = obj["findings"];
  if (!Array.isArray(findings)) {
    fail("findings is not a list");
  }
  return { score, findings: findings.map((item) => parseFinding(item)) };
}
