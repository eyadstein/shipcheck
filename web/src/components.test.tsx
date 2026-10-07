import { renderToString } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { FindingsTable, PAGE_SIZE } from "./components/FindingsTable";
import { ProjectList } from "./components/ProjectList";
import { ScoreBadge } from "./components/ScoreBadge";
import { ScoreChart } from "./components/ScoreChart";
import { makeFinding } from "./test-data";

function rowCount(html: string): number {
  return (html.match(/<tr[\s>]/g) ?? []).length;
}

describe("ScoreBadge", () => {
  it("shows the score with a text label, not just a color", () => {
    const html = renderToString(<ScoreBadge score={95} />);
    expect(html).toContain("score-good");
    expect(html).toContain("Good");
    expect(renderToString(<ScoreBadge score={10} />)).toContain("Failing");
  });
});

describe("ScoreChart", () => {
  it("describes the trend for screen readers", () => {
    const html = renderToString(<ScoreChart scores={[40, 70]} labels={["a", "b"]} />);
    expect(html).toContain('aria-label="Score up 30 points, from 40 to 70 over 2 scans"');
    expect(html).toContain("<circle");
  });
});

describe("FindingsTable", () => {
  it("explains an empty result", () => {
    expect(renderToString(<FindingsTable findings={[]} />)).toContain("No findings match");
  });

  it("renders one row per finding plus the header", () => {
    const html = renderToString(
      <FindingsTable findings={[makeFinding(), makeFinding({ ruleId: "SEC-102" })]} />,
    );
    expect(rowCount(html)).toBe(3);
    expect(html).toContain("SEC-102");
  });

  it("offers the fix only when there is one", () => {
    const withFix = renderToString(<FindingsTable findings={[makeFinding({ fix: "Use a parameterized query" })]} />);
    expect(withFix).toContain("How to fix");
    const without = renderToString(<FindingsTable findings={[makeFinding()]} />);
    expect(without).not.toContain("How to fix");
  });

  it("shows project level findings without a line number", () => {
    const html = renderToString(<FindingsTable findings={[makeFinding({ file: "(project)", line: 0 })]} />);
    expect(html).toContain("(project)");
    expect(html).not.toContain("(project):0");
  });

  it("pages long lists", () => {
    const many = Array.from({ length: PAGE_SIZE + 50 }, (_, index) => makeFinding({ line: index + 1 }));
    const html = renderToString(<FindingsTable findings={many} />);
    expect(rowCount(html)).toBe(PAGE_SIZE + 1);
    expect(html).toContain("Show 50 more");
  });
});

describe("ProjectList", () => {
  it("explains how to get started when empty", () => {
    expect(renderToString(<ProjectList projects={[]} />)).toContain("No scans stored yet");
  });

  it("links each project to its page", () => {
    const html = renderToString(
      <ProjectList
        projects={[{ project: "a/b", latestScore: 80, lastScanAt: "2026-10-05T10:00:00Z", scanCount: 2 }]}
      />,
    );
    expect(html).toContain('href="#/p/a%2Fb"');
  });
});
