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
  const shownRoute = useRef<string | null>(null);

  useEffect(() => {
    document.title = titleFor(route);
    // Compare against the page shown before, so running the effect twice
    // in development mode never counts as a navigation.
    const key = JSON.stringify(route);
    const changed = shownRoute.current !== null && shownRoute.current !== key;
    shownRoute.current = key;
    if (changed) {
      // Move keyboard and screen reader focus to the new page content.
      document.getElementById("main")?.focus();
      window.scrollTo(0, 0);
    }
  }, [route]);

  return <Layout>{renderRoute(route)}</Layout>;
}
