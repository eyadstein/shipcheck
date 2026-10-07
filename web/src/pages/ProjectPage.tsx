import { fetchScans } from "../api";
import { ScanTable } from "../components/ScanTable";
import { ScoreChart } from "../components/ScoreChart";
import { Remote } from "../components/States";
import { formatDate } from "../format";
import { HOME_HREF } from "../router";
import { oldestFirst } from "../sort";
import type { ScanSummary } from "../types";
import { useAsync } from "../useAsync";

const HISTORY_LIMIT = 50;

function History({ scans }: { scans: readonly ScanSummary[] }) {
  if (scans.length === 0) {
    return <p className="state">No scans stored for this project yet.</p>;
  }
  const ordered = oldestFirst(scans);
  return (
    <>
      <section aria-labelledby="trend-title">
        <h2 id="trend-title">Score history</h2>
        <ScoreChart
          scores={ordered.map((scan) => scan.score)}
          labels={ordered.map((scan) => formatDate(scan.createdAt))}
        />
      </section>
      <section aria-labelledby="scans-title">
        <h2 id="scans-title">Scans</h2>
        <ScanTable scans={[...ordered].reverse()} />
      </section>
    </>
  );
}

export function ProjectPage({ project }: { project: string }) {
  const state = useAsync((signal) => fetchScans(project, HISTORY_LIMIT, signal), [project]);
  return (
    <>
      <p className="crumbs">
        <a href={HOME_HREF}>Projects</a>
      </p>
      <h1 className="page-title">{project}</h1>
      <Remote state={state} what="scan history">
        {(scans) => <History scans={scans} />}
      </Remote>
    </>
  );
}
