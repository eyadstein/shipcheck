export type Route =
  | { page: "projects" }
  | { page: "project"; project: string }
  | { page: "scan"; id: string }
  | { page: "missing" };

export const HOME_HREF = "#/";

export function projectHref(project: string): string {
  return `#/p/${encodeURIComponent(project)}`;
}

export function scanHref(id: string): string {
  return `#/scan/${encodeURIComponent(id)}`;
}

function decode(part: string): string | null {
  try {
    return decodeURIComponent(part);
  } catch {
    return null;
  }
}

export function parseRoute(hash: string): Route {
  const parts = hash
    .replace(/^#/, "")
    .split("/")
    .filter((part) => part !== "");
  if (parts.length === 0) {
    return { page: "projects" };
  }
  const [kind, value, ...rest] = parts;
  if (value === undefined || rest.length > 0) {
    return { page: "missing" };
  }
  const decoded = decode(value);
  if (decoded === null) {
    return { page: "missing" };
  }
  if (kind === "p") return { page: "project", project: decoded };
  if (kind === "scan") return { page: "scan", id: decoded };
  return { page: "missing" };
}

export function titleFor(route: Route): string {
  switch (route.page) {
    case "projects":
      return "Projects | Shipcheck";
    case "project":
      return `${route.project} | Shipcheck`;
    case "scan":
      return "Scan | Shipcheck";
    case "missing":
      return "Not found | Shipcheck";
  }
}
