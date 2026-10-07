import { describe, expect, it } from "vitest";
import {
  ApiError,
  parseFinding,
  parseProjectList,
  parseScanDetail,
  parseScanList,
  scansPath,
} from "./api";

const finding = {
  rule_id: "SEC-101",
  category: "security",
  severity: "high",
  message: "m",
  file: "a.js",
  line: 3,
  fix: null,
};

const scan = {
  id: "7d4c3f2e-0000-4000-8000-000000000001",
  project: "a/b",
  git_ref: "main",
  score: 90,
  created_at: "2026-10-05T10:00:00Z",
  finding_count: 1,
};

describe("parseFinding", () => {
  it("maps snake case fields to camel case", () => {
    expect(parseFinding(finding)).toEqual({
      ruleId: "SEC-101",
      category: "security",
      severity: "high",
      message: "m",
      file: "a.js",
      line: 3,
      fix: null,
    });
  });

  it("keeps a fix text", () => {
    expect(parseFinding({ ...finding, fix: "do this" }).fix).toBe("do this");
  });

  it("rejects unknown severities and categories", () => {
    expect(() => parseFinding({ ...finding, severity: "catastrophic" })).toThrow(ApiError);
    expect(() => parseFinding({ ...finding, category: "other" })).toThrow(ApiError);
  });

  it("rejects missing or mistyped fields", () => {
    expect(() => parseFinding({ ...finding, line: "3" })).toThrow(/line/);
    expect(() => parseFinding({ ...finding, message: undefined })).toThrow(/message/);
    expect(() => parseFinding("nope")).toThrow(ApiError);
  });
});

describe("scan parsers", () => {
  it("parses a scan list", () => {
    const parsed = parseScanList([scan]);
    expect(parsed).toHaveLength(1);
    expect(parsed[0]?.gitRef).toBe("main");
    expect(parsed[0]?.findingCount).toBe(1);
  });

  it("accepts a missing git ref", () => {
    expect(parseScanList([{ ...scan, git_ref: null }])[0]?.gitRef).toBeNull();
  });

  it("rejects a response that is not a list", () => {
    expect(() => parseScanList({ error: "x" })).toThrow(ApiError);
  });

  it("parses a scan with its findings", () => {
    const detail = parseScanDetail({ ...scan, findings: [finding, finding] });
    expect(detail.findings).toHaveLength(2);
    expect(detail.project).toBe("a/b");
  });

  it("rejects a scan without findings", () => {
    expect(() => parseScanDetail(scan)).toThrow(/findings/);
  });
});

describe("parseProjectList", () => {
  it("parses projects", () => {
    const parsed = parseProjectList([
      { project: "a/b", latest_score: 80, last_scan_at: "2026-10-05T10:00:00Z", scan_count: 3 },
    ]);
    expect(parsed).toEqual([
      { project: "a/b", latestScore: 80, lastScanAt: "2026-10-05T10:00:00Z", scanCount: 3 },
    ]);
  });

  it("rejects fractional counts", () => {
    expect(() =>
      parseProjectList([{ project: "a", latest_score: 1.5, last_scan_at: "x", scan_count: 1 }]),
    ).toThrow(ApiError);
  });
});

describe("scansPath", () => {
  it("builds the query string", () => {
    expect(scansPath(null, 50)).toBe("/api/v1/scans?limit=50");
  });

  it("encodes the project name", () => {
    const path = scansPath("owner/repo name", 10);
    expect(path).toContain("project=owner%2Frepo+name");
    expect(path).toContain("limit=10");
  });
});
