import { describe, expect, it } from "vitest";
import { oldestFirst, sortProjects } from "./sort";
import type { ProjectSummary, ScanSummary } from "./types";

function project(name: string, latestScore: number): ProjectSummary {
  return { project: name, latestScore, lastScanAt: "2026-10-05T10:00:00Z", scanCount: 1 };
}

function scan(id: string, createdAt: string): ScanSummary {
  return { id, project: "a/b", gitRef: null, score: 50, createdAt, findingCount: 0 };
}

describe("sortProjects", () => {
  it("puts the lowest score first and breaks ties by name", () => {
    const sorted = sortProjects([project("b", 90), project("c", 20), project("a", 90)]);
    expect(sorted.map((item) => item.project)).toEqual(["c", "a", "b"]);
  });
});

describe("oldestFirst", () => {
  it("orders scans by date without changing the input", () => {
    const input = [scan("new", "2026-10-05T10:00:00Z"), scan("old", "2026-10-01T10:00:00Z")];
    expect(oldestFirst(input).map((item) => item.id)).toEqual(["old", "new"]);
    expect(input[0]?.id).toBe("new");
  });
});
