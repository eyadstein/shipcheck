export interface ChartBox {
  width: number;
  height: number;
  left: number;
  right: number;
  top: number;
  bottom: number;
}

export interface ChartPoint {
  x: number;
  y: number;
  score: number;
}

export const CHART_BOX: ChartBox = { width: 640, height: 200, left: 40, right: 16, top: 16, bottom: 28 };

function clampScore(score: number): number {
  return Math.min(100, Math.max(0, score));
}

/** Vertical position of a score. 100 sits at the top of the plot area. */
export function scoreY(score: number, box: ChartBox): number {
  return box.top + (box.height - box.top - box.bottom) * (1 - clampScore(score) / 100);
}

export function chartPoints(scores: readonly number[], box: ChartBox): ChartPoint[] {
  const innerWidth = box.width - box.left - box.right;
  const last = scores.length - 1;
  return scores.map((score, index) => ({
    x: last === 0 ? box.left + innerWidth / 2 : box.left + (innerWidth * index) / last,
    y: scoreY(score, box),
    score,
  }));
}

function round(value: number): number {
  return Math.round(value * 10) / 10;
}

export function linePath(points: readonly ChartPoint[]): string {
  return points
    .map((point, index) => `${index === 0 ? "M" : "L"}${round(point.x)} ${round(point.y)}`)
    .join(" ");
}

/** A sentence for screen readers that sums up the chart. */
export function describeTrend(scores: readonly number[]): string {
  const first = scores[0];
  const last = scores[scores.length - 1];
  if (first === undefined || last === undefined) {
    return "No scans yet";
  }
  if (scores.length === 1) {
    return `One scan, score ${last}`;
  }
  const change = last - first;
  if (change === 0) {
    return `Score unchanged at ${last} over ${scores.length} scans`;
  }
  const direction = change > 0 ? "up" : "down";
  return `Score ${direction} ${Math.abs(change)} points, from ${first} to ${last} over ${scores.length} scans`;
}
