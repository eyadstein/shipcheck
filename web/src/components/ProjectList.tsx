import { formatDate, timeAgo } from "../format";
import { projectHref } from "../router";
import type { ProjectSummary } from "../types";
import { ScoreBadge } from "./ScoreBadge";

export function ProjectList({ projects }: { projects: readonly ProjectSummary[] }) {
  if (projects.length === 0) {
    return (
      <p className="state">
        No scans stored yet. Send a report to the API or run the GitHub Action.
      </p>
    );
  }
  return (
    <div className="table-wrap">
      <table className="table">
        <caption className="visually-hidden">Projects, lowest score first</caption>
        <thead>
          <tr>
            <th scope="col">Project</th>
            <th scope="col">Latest score</th>
            <th scope="col" className="num">
              Scans
            </th>
            <th scope="col">Last scan</th>
          </tr>
        </thead>
        <tbody>
          {projects.map((project) => (
            <tr key={project.project}>
              <td>
                <a href={projectHref(project.project)}>{project.project}</a>
              </td>
              <td>
                <ScoreBadge score={project.latestScore} />
              </td>
              <td className="num">{project.scanCount}</td>
              <td>
                <time dateTime={project.lastScanAt} title={formatDate(project.lastScanAt)}>
                  {timeAgo(project.lastScanAt)}
                </time>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
