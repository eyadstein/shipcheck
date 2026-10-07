import { formatDate } from "../format";
import { scanHref } from "../router";
import type { ScanSummary } from "../types";
import { ScoreBadge } from "./ScoreBadge";

export function ScanTable({ scans }: { scans: readonly ScanSummary[] }) {
  return (
    <div className="table-wrap">
      <table className="table">
        <caption className="visually-hidden">Scans, newest first</caption>
        <thead>
          <tr>
            <th scope="col">Date</th>
            <th scope="col">Ref</th>
            <th scope="col">Score</th>
            <th scope="col" className="num">
              Findings
            </th>
          </tr>
        </thead>
        <tbody>
          {scans.map((scan) => (
            <tr key={scan.id}>
              <td>
                <a href={scanHref(scan.id)}>{formatDate(scan.createdAt)}</a>
              </td>
              <td>{scan.gitRef ?? "none"}</td>
              <td>
                <ScoreBadge score={scan.score} />
              </td>
              <td className="num">{scan.findingCount}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
