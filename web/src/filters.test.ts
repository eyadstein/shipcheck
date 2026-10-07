import { describe, expect, it } from "vitest";
import {
  NO_FILTER,
  applyFilter,
  categoryCounts,
  isFiltered,
  parseCategory,
  parseSeverity,
  severityCounts,
} from "./filters";
import { makeFinding } from "./test-data";

const findings = [
  makeFinding({ ruleId: "LEGAL-001", category: "legal", severity: "medium", file: "(project)", line: 0, message: "No privacy policy" }),
  makeFinding({ ruleId: "SEC-102", severity: "critical", file: "b.js", line: 2 }),
  makeFinding({ ruleId: "SEC-101", severity: "high", file: "a.js", line: 9 }),
  makeFinding({ ruleId: "DES-201", category: "design", severity: "low", file: "a.css", line: 1, message: "Gradient text" }),
  makeFinding({ ruleId: "SEC-103", severity: "high", file: "a.js", line: 3 }),
];

describe("applyFilter", () => {
  it("sorts by severity, then file, then line", () => {
    const ids = applyFilter(findings, NO_FILTER).map((finding) => finding.ruleId);
    expect(ids).toEqual(["SEC-102", "SEC-103", "SEC-101", "LEGAL-001", "DES-201"]);
  });

  it("does not change the input", () => {
    const before = findings.map((finding) => finding.ruleId);
    applyFilter(findings, NO_FILTER);
    expect(findings.map((finding) => finding.ruleId)).toEqual(before);
  });

  it("filters by category and severity", () => {
    expect(applyFilter(findings, { ...NO_FILTER, category: "design" })).toHaveLength(1);
    expect(applyFilter(findings, { ...NO_FILTER, severity: "high" })).toHaveLength(2);
    expect(applyFilter(findings, { category: "security", severity: "critical", query: "" })).toHaveLength(1);
  });

  it("searches rule ids, files and messages without caring about case", () => {
    expect(applyFilter(findings, { ...NO_FILTER, query: "gradient" })).toHaveLength(1);
    expect(applyFilter(findings, { ...NO_FILTER, query: "A.JS" })).toHaveLength(2);
    expect(applyFilter(findings, { ...NO_FILTER, query: "legal-001" })).toHaveLength(1);
    expect(applyFilter(findings, { ...NO_FILTER, query: "   " })).toHaveLength(5);
  });
});

describe("helpers", () => {
  it("counts by severity and category", () => {
    expect(severityCounts(findings)).toEqual({ info: 0, low: 1, medium: 1, high: 2, critical: 1 });
    expect(categoryCounts(findings)).toEqual({ legal: 1, security: 3, design: 1 });
  });

  it("knows when a filter is active", () => {
    expect(isFiltered(NO_FILTER)).toBe(false);
    expect(isFiltered({ ...NO_FILTER, query: " x " })).toBe(true);
    expect(isFiltered({ ...NO_FILTER, severity: "low" })).toBe(true);
  });

  it("falls back to all for unknown values", () => {
    expect(parseCategory("design")).toBe("design");
    expect(parseCategory("nope")).toBe("all");
    expect(parseSeverity("critical")).toBe("critical");
    expect(parseSeverity("")).toBe("all");
  });
});
