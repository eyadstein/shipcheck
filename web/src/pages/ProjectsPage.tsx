import { fetchProjects } from "../api";
import { ProjectList } from "../components/ProjectList";
import { Remote } from "../components/States";
import { sortProjects } from "../sort";
import { useAsync } from "../useAsync";

export function ProjectsPage() {
  const state = useAsync(fetchProjects, []);
  return (
    <>
      <h1 className="page-title">Projects</h1>
      <Remote state={state} what="projects">
        {(projects) => <ProjectList projects={sortProjects(projects)} />}
      </Remote>
    </>
  );
}
