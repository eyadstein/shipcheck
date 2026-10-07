import { useEffect, useRef } from "react";
import { Layout } from "./components/Layout";
import { NotFoundPage } from "./pages/NotFoundPage";
import { ProjectPage } from "./pages/ProjectPage";
import { ProjectsPage } from "./pages/ProjectsPage";
import { ScanPage } from "./pages/ScanPage";
import { titleFor, type Route } from "./router";
import { useRoute } from "./useRoute";

function renderRoute(route: Route) {
  switch (route.page) {
    case "projects":
      return <ProjectsPage />;
    case "project":
      return <ProjectPage key={route.project} project={route.project} />;
    case "scan":
      return <ScanPage key={route.id} id={route.id} />;
    case "missing":
      return <NotFoundPage />;
  }
}

export function App() {
  const route = useRoute();
  const firstRender = useRef(true);

  useEffect(() => {
    document.title = titleFor(route);
    if (firstRender.current) {
      firstRender.current = false;
      return;
    }
    // Move keyboard and screen reader focus to the new page content.
    document.getElementById("main")?.focus();
    window.scrollTo(0, 0);
  }, [route]);

  return <Layout>{renderRoute(route)}</Layout>;
}
