import { describe, expect, it } from "vitest";
import { capitalize, formatDate, formatLocation, scoreTone, severityRank, timeAgo } from "./format";

describe("scoreTone", () => {
  it("uses the documented thresholds", () => {
    expect(scoreTone(100)).toBe("good");
    expect(scoreTone(90)).toBe("good");
    expect(scoreTone(89)).toBe("fair");
    expect(scoreTone(70)).toBe("fair");
    expect(scoreTone(69)).toBe("poor");
    expect(scoreTone(40)).toBe("poor");
    expect(scoreTone(39)).toBe("bad");
    expect(scoreTone(0)).toBe("bad");
  });
});

describe("severityRank", () => {
  it("orders from info up to critical", () => {
    expect(severityRank("critical")).toBeGreaterThan(severityRank("high"));
    expect(severityRank("high")).toBeGreaterThan(severityRank("medium"));
    expect(severityRank("low")).toBeGreaterThan(severityRank("info"));
  });
});

describe("formatLocation", () => {
  it("adds the line when there is one", () => {
    expect(formatLocation({ file: "a.js", line: 4 })).toBe("a.js:4");
  });

  it("leaves project level findings alone", () => {
    expect(formatLocation({ file: "(project)", line: 0 })).toBe("(project)");
  });
});

describe("timeAgo", () => {
  const now = Date.parse("2026-10-05T12:00:00Z");
  const ago = (seconds: number): string => new Date(now - seconds * 1000).toISOString();

  it("describes recent and older times", () => {
    expect(timeAgo(ago(5), now)).toBe("just now");
    expect(timeAgo(ago(60), now)).toBe("1 minute ago");
    expect(timeAgo(ago(5 * 60), now)).toBe("5 minutes ago");
    expect(timeAgo(ago(3 * 3600), now)).toBe("3 hours ago");
    expect(timeAgo(ago(2 * 86400), now)).toBe("2 days ago");
    expect(timeAgo(ago(65 * 86400), now)).toBe("2 months ago");
    expect(timeAgo(ago(400 * 86400), now)).toBe("1 year ago");
  });

  it("never reports the future as negative", () => {
    expect(timeAgo(ago(-500), now)).toBe("just now");
  });

  it("returns invalid input unchanged", () => {
    expect(timeAgo("not a date", now)).toBe("not a date");
  });
});

describe("formatDate and capitalize", () => {
  it("formats valid dates and keeps invalid ones", () => {
    expect(formatDate("2026-10-05T12:00:00Z")).toContain("2026");
    expect(formatDate("garbage")).toBe("garbage");
  });

  it("capitalizes the first letter", () => {
    expect(capitalize("security")).toBe("Security");
    expect(capitalize("")).toBe("");
  });
});
