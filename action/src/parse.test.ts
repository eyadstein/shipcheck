import { describe, expect, it } from "vitest";
import { parseReport } from "./parse.ts";

const finding = {
  rule_id: "SEC-101",
  category: "security",
  severity: "high",
  message: "m",
  file: "a.js",
  line: 3,
  fix: null,
};

function report(overrides: Record<string, unknown> = {}): string {
  return JSON.stringify({ score: 90, findings: [finding], ...overrides });
}

describe("parseReport", () => {
  it("reads the JSON printed by the scanner", () => {
    const parsed = parseReport(report());
    expect(parsed.score).toBe(90);
    expect(parsed.findings).toEqual([
      {
        ruleId: "SEC-101",
        category: "security",
        severity: "high",
        message: "m",
        file: "a.js",
        line: 3,
        fix: null,
      },
    ]);
  });

  it("ignores a byte order mark", () => {
    expect(parseReport(`\uFEFF${report()}`).score).toBe(90);
  });

  it("rejects text that is not JSON", () => {
    expect(() => parseReport("not json")).toThrow(/not valid JSON/);
  });

  it("rejects unknown severities and categories", () => {
    const badSeverity = report({ findings: [{ ...finding, severity: "catastrophic" }] });
    const badCategory = report({ findings: [{ ...finding, category: "other" }] });
    expect(() => parseReport(badSeverity)).toThrow(/severity/);
    expect(() => parseReport(badCategory)).toThrow(/category/);
  });

  it("rejects mistyped fields", () => {
    expect(() => parseReport(report({ findings: [{ ...finding, line: "3" }] }))).toThrow(/line/);
    expect(() => parseReport(report({ findings: ["nope"] }))).toThrow(/not an object/);
    expect(() => parseReport(report({ score: 1.5 }))).toThrow(/score/);
  });

  it("rejects scores outside 0 to 100", () => {
    expect(() => parseReport(report({ score: 101 }))).toThrow(/outside/);
    expect(() => parseReport(report({ score: -1 }))).toThrow(/outside/);
  });

  it("rejects reports without a findings list", () => {
    expect(() => parseReport(JSON.stringify({ score: 90 }))).toThrow(/findings/);
  });
});
