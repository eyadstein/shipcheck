import { CHART_BOX, chartPoints, describeTrend, linePath, scoreY } from "../trend";

const GRID_LINES = [0, 50, 100];

interface ScoreChartProps {
  scores: readonly number[];
  labels: readonly string[];
}

export function ScoreChart({ scores, labels }: ScoreChartProps) {
  const box = CHART_BOX;
  const points = chartPoints(scores, box);
  const description = describeTrend(scores);
  return (
    <figure className="chart">
      <svg viewBox={`0 0 ${box.width} ${box.height}`} role="img" aria-label={description}>
        {GRID_LINES.map((value) => {
          const y = scoreY(value, box);
          return (
            <g key={value}>
              <line x1={box.left} x2={box.width - box.right} y1={y} y2={y} className="chart-grid" />
              <text x={box.left - 8} y={y + 4} textAnchor="end" className="chart-axis">
                {value}
              </text>
            </g>
          );
        })}
        <path d={linePath(points)} className="chart-line" fill="none" />
        {points.map((point, index) => (
          <circle key={index} cx={point.x} cy={point.y} r={4} className="chart-dot">
            <title>{`${labels[index] ?? ""}: score ${point.score}`}</title>
          </circle>
        ))}
      </svg>
      <figcaption>{description}</figcaption>
    </figure>
  );
}
