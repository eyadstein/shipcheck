import { describe, expect, it } from "vitest";
import { CHART_BOX, chartPoints, describeTrend, linePath, scoreY } from "./trend";

const box = CHART_BOX;
const bottom = box.height - box.bottom;

describe("scoreY", () => {
  it("puts 100 at the top and 0 at the bottom of the plot", () => {
    expect(scoreY(100, box)).toBe(box.top);
    expect(scoreY(0, box)).toBe(bottom);
  });

  it("clamps scores outside the range", () => {
    expect(scoreY(250, box)).toBe(box.top);
    expect(scoreY(-5, box)).toBe(bottom);
  });
});

describe("chartPoints", () => {
  it("spreads points across the width", () => {
    const [first, second, third] = chartPoints([0, 50, 100], box);
    expect(first?.x).toBe(box.left);
    expect(third?.x).toBe(box.width - box.right);
    expect(second?.x).toBeCloseTo((box.left + box.width - box.right) / 2);
  });

  it("centers a single point", () => {
    const [only] = chartPoints([80], box);
    expect(only?.x).toBeCloseTo((box.left + box.width - box.right) / 2);
  });

  it("returns nothing for no scores", () => {
    expect(chartPoints([], box)).toEqual([]);
  });
});

describe("linePath", () => {
  it("starts with a move and continues with lines", () => {
    const path = linePath(chartPoints([0, 100], box));
    expect(path.startsWith("M")).toBe(true);
    expect(path.match(/L/g)).toHaveLength(1);
  });

  it("is empty without points", () => {
    expect(linePath([])).toBe("");
  });
});

describe("describeTrend", () => {
  it("describes every shape of history", () => {
    expect(describeTrend([])).toBe("No scans yet");
    expect(describeTrend([70])).toBe("One scan, score 70");
    expect(describeTrend([40, 70])).toBe("Score up 30 points, from 40 to 70 over 2 scans");
    expect(describeTrend([90, 60, 50])).toBe("Score down 40 points, from 90 to 50 over 3 scans");
    expect(describeTrend([50, 80, 50])).toBe("Score unchanged at 50 over 3 scans");
  });
});
