import { useMemo, useState } from "react";
import { fetchScan } from "../api";
import { FilterBar } from "../components/FilterBar";
import { FindingsTable } from "../components/FindingsTable";
import { ScoreBadge } from "../components/ScoreBadge";
import { SeverityTag } from "../components/SeverityTag";
import { Remote } from "../components/States";
import { NO_FILTER, applyFilter, categoryCounts, severityCounts, type FindingFilter } from "../filters";
import { capitalize, formatDate } from "../format";
import { HOME_HREF, projectHref } from "../router";
import { CATEGORIES, SEVERITIES, type ScanDetail } from "../types";
import { useAsync } from "../useAsync";

function ScanView({ scan }: { scan: ScanDetail }) {
  const [filter, setFilter] = useState<FindingFilter>(NO_FILTER);
  const shown = useMemo(() => applyFilter(scan.findings, filter), [scan.findings, filter]);
  const severities = severityCounts(scan.findings);
  const categories = categoryCounts(scan.findings);
  return (
    <>
      <p className="crumbs">
        <a href={HOME_HREF}>Projects</a> / <a href={projectHref(scan.project)}>{scan.project}</a>
      </p>
      <h1 className="page-title">{`Scan from ${formatDate(scan.createdAt)}`}</h1>
      <div className="summary">
        <ScoreBadge score={scan.score} />
        <dl className="facts">
          <div>
            <dt>Ref</dt>
            <dd>{scan.gitRef ?? "none"}</dd>
          </div>
          <div>
            <dt>Findings</dt>
            <dd>{scan.findingCount}</dd>
          </div>
          {CATEGORIES.map((category) => (
            <div key={category}>
              <dt>{capitalize(category)}</dt>
              <dd>{categories[category]}</dd>
            </div>
          ))}
        </dl>
        <ul className="tags">
          {[...SEVERITIES]
            .reverse()
            .filter((severity) => severities[severity] > 0)
            .map((severity) => (
              <li key={severity}>
                <SeverityTag severity={severity} /> {severities[severity]}
              </li>
            ))}
        </ul>
      </div>
      <h2>Findings</h2>
      <FilterBar filter={filter} onChange={setFilter} total={scan.findings.length} shown={shown.length} />
      <FindingsTable findings={shown} />
    </>
  );
}

export function ScanPage({ id }: { id: string }) {
  const state = useAsync((signal) => fetchScan(id, signal), [id]);
  return (
    <Remote state={state} what="scan">
      {(scan) => <ScanView scan={scan} />}
    </Remote>
  );
}
