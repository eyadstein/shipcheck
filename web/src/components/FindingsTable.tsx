import { useState } from "react";
import { capitalize, formatLocation } from "../format";
import type { Finding } from "../types";
import { SeverityTag } from "./SeverityTag";

export const PAGE_SIZE = 100;

export function FindingsTable({ findings }: { findings: readonly Finding[] }) {
  const [limit, setLimit] = useState(PAGE_SIZE);
  if (findings.length === 0) {
    return <p className="state">No findings match the current filters.</p>;
  }
  const visible = findings.slice(0, limit);
  const hidden = findings.length - visible.length;
  return (
    <>
      <div className="table-wrap">
        <table className="table">
          <caption className="visually-hidden">Findings, most severe first</caption>
          <thead>
            <tr>
              <th scope="col">Severity</th>
              <th scope="col">Rule</th>
              <th scope="col">Location</th>
              <th scope="col">Problem</th>
            </tr>
          </thead>
          <tbody>
            {visible.map((finding, index) => (
              <tr key={`${finding.ruleId}:${finding.file}:${finding.line}:${index}`}>
                <td>
                  <SeverityTag severity={finding.severity} />
                </td>
                <td>
                  <code>{finding.ruleId}</code>
                  <div className="muted">{capitalize(finding.category)}</div>
                </td>
                <td>
                  <code>{formatLocation(finding)}</code>
                </td>
                <td>
                  {finding.message}
                  {finding.fix === null ? null : (
                    <details className="fix">
                      <summary>How to fix</summary>
                      <p>{finding.fix}</p>
                    </details>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      {hidden > 0 ? (
        <p>
          <button
            type="button"
            className="button button-quiet"
            onClick={() => setLimit(limit + PAGE_SIZE)}
          >
            {`Show ${Math.min(PAGE_SIZE, hidden)} more`}
          </button>
        </p>
      ) : null}
    </>
  );
}
