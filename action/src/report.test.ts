import { describe, expect, it } from "vitest";
import {
  MARKER,
  code,
  normalizeScanPath,
  parseFlag,
  parseThreshold,
  plain,
  renderComment,
  scoreTone,
} from "./report.ts";
import type { RenderOptions } from "./report.ts";
import type { Finding } from "./types.ts";

const OPTIONS: RenderOptions = {
  failUnder: 0,
  scanPath: "",
  repository: "o/r",
  sha: "abc123",
  serverUrl: "https://github.com",
};

function finding(overrides: Partial<Finding> = {}): Finding {
  return {
    ruleId: "SEC-101",
    category: "security",
    severity: "high",
    message: "User input reaches a SQL query (CWE-89).",
    file: "src/db.js",
    line: 10,
    fix: null,
    ...overrides,
  };
}

describe("parseThreshold", () => {
  it("accepts whole numbers from 0 to 100", () => {
    expect(parseThreshold("")).toBe(0);
    expect(parseThreshold(" 80 ")).toBe(80);
    expect(parseThreshold("100")).toBe(100);
  });

  it("rejects everything else", () => {
    for (const bad of ["-1", "101", "abc", "8.5", "0x10", "80%"]) {
      expect(() => parseThreshold(bad)).toThrow(/fail-under/);
    }
  });
});

describe("parseFlag", () => {
  it("reads true and false, with a fallback for empty input", () => {
    expect(parseFlag("", true)).toBe(true);
    expect(parseFlag("", false)).toBe(false);
    expect(parseFlag("TRUE", false)).toBe(true);
    expect(parseFlag(" false ", true)).toBe(false);
    expect(() => parseFlag("yes", true)).toThrow(/true or false/);
  });
});

describe("normalizeScanPath", () => {
  it("makes paths relative to the repository root", () => {
    expect(normalizeScanPath(".")).toBe("");
    expect(normalizeScanPath("./")).toBe("");
    expect(normalizeScanPath("")).toBe("");
    expect(normalizeScanPath("./web/")).toBe("web");
    expect(normalizeScanPath("web\\src")).toBe("web/src");
    expect(normalizeScanPath("/abs/path")).toBe("/abs/path");
  });
});

describe("text helpers", () => {
  it("builds code spans that survive backticks", () => {
    expect(code("a.js")).toBe("`a.js`");
    expect(code("a`b")).toBe("``a`b``");
    expect(code("`x")).toBe("`` `x ``");
    expect(code("a\nb")).toBe("`a b`");
  });

  it("flattens lines, neutralizes mentions and escapes tags", () => {
    expect(plain("a\nb @octocat <b>")).toBe("a b @\u200boctocat &lt;b>");
  });

  it("names the score bands", () => {
    expect(scoreTone(90)).toBe("Good");
    expect(scoreTone(89)).toBe("Fair");
    expect(scoreTone(70)).toBe("Fair");
    expect(scoreTone(69)).toBe("Poor");
    expect(scoreTone(40)).toBe("Poor");
    expect(scoreTone(39)).toBe("Failing");
  });
});

describe("renderComment", () => {
  it("starts with the marker and a score heading", () => {
    const text = renderComment({ score: 72, findings: [] }, OPTIONS);
    expect(text.startsWith(`${MARKER}\n## Shipcheck: 72/100 (Fair)`)).toBe(true);
    expect(text).toContain("No findings.");
  });

  it("states whether the threshold was met", () => {
    const report = { score: 72, findings: [] };
    expect(renderComment(report, { ...OPTIONS, failUnder: 80 })).toContain(
      "Failed: the score is below the required 80.",
    );
    expect(renderComment(report, { ...OPTIONS, failUnder: 70 })).toContain(
      "Passed: the score is at or above the required 70.",
    );
    expect(renderComment(report, OPTIONS)).toContain("No minimum score is required.");
  });

  it("lists the most severe findings first", () => {
    const report = {
      score: 50,
      findings: [
        finding({ ruleId: "DES-201", severity: "low", file: "a.css", line: 1 }),
        finding({ ruleId: "SEC-102", severity: "critical", file: "b.js", line: 2 }),
        finding({ ruleId: "SEC-101", severity: "high", file: "a.js", line: 9 }),
      ],
    };
    const text = renderComment(report, OPTIONS);
    const order = ["SEC-102", "SEC-101", "DES-201"].map((id) => text.indexOf(id));
    expect(order.every((index) => index > 0)).toBe(true);
    expect(order).toEqual([...order].sort((a, b) => a - b));
  });

  it("summarizes the counts", () => {
    const report = {
      score: 50,
      findings: [
        finding({ severity: "critical" }),
        finding({ severity: "high", category: "legal" }),
        finding({ severity: "low", category: "design" }),
      ],
    };
    const text = renderComment(report, OPTIONS);
    expect(text).toContain("3 findings: 1 critical, 1 high, 1 low.");
    expect(text).toContain("| Security | 1 |");
    expect(text).toContain("| Legal | 1 |");
    expect(text).toContain("| Design | 1 |");
  });

  it("links findings to the file at the commit", () => {
    const report = { score: 90, findings: [finding({ file: "src/a b.ts", line: 7 })] };
    const text = renderComment(report, { ...OPTIONS, scanPath: "web" });
    expect(text).toContain(
      "[`src/a b.ts:7`](https://github.com/o/r/blob/abc123/web/src/a%20b.ts#L7)",
    );
  });

  it("does not link project level findings or unsafe paths", () => {
    const projectLevel = renderComment(
      { score: 90, findings: [finding({ file: "(project)", line: 0 })] },
      OPTIONS,
    );
    expect(projectLevel).toContain("`(project)`");
    expect(projectLevel).not.toContain("](");
    const report = { score: 90, findings: [finding()] };
    expect(renderComment(report, { ...OPTIONS, scanPath: "/home/runner" })).not.toContain("](");
    expect(renderComment(report, { ...OPTIONS, scanPath: "../x" })).not.toContain("](");
    expect(renderComment(report, { ...OPTIONS, sha: null })).not.toContain("](");
  });

  it("keeps odd file names inside a code span", () => {
    const report = { score: 90, findings: [finding({ file: "a|b`c.js", line: 3 })] };
    const text = renderComment(report, { ...OPTIONS, sha: null });
    expect(text).toContain("``a|b`c.js:3``");
  });

  it("shows the suggested fix", () => {
    const report = { score: 90, findings: [finding({ fix: "Use parameters." })] };
    expect(renderComment(report, OPTIONS)).toContain("\n  - Fix: Use parameters.");
  });

  it("neutralizes mentions in messages", () => {
    const report = { score: 90, findings: [finding({ message: "ping @octocat" })] };
    const text = renderComment(report, OPTIONS);
    expect(text).toContain("@\u200boctocat");
    expect(text).not.toContain("@octocat");
  });

  it("limits the number of listed findings", () => {
    const many = Array.from({ length: 30 }, (_, index) =>
      finding({ file: `f${index}.js`, line: index + 1 }),
    );
    const text = renderComment({ score: 10, findings: many }, { ...OPTIONS, maxFindings: 5 });
    const listed = text.split("\n").filter((line) => line.startsWith("- **"));
    expect(listed).toHaveLength(5);
    expect(text).toContain("25 more findings not shown.");
  });

  it("never exceeds the comment size limit", () => {
    const huge = Array.from({ length: 40 }, (_, index) =>
      finding({ file: `f${index}.js`, line: 1, message: "x".repeat(5000) }),
    );
    const text = renderComment({ score: 0, findings: huge }, OPTIONS);
    expect(text.length).toBeLessThanOrEqual(60_000);
    expect(text.startsWith(MARKER)).toBe(true);
    expect(text).toContain("more findings not shown.");
  });
});
