import { HOME_HREF } from "../router";

export function NotFoundPage() {
  return (
    <>
      <h1 className="page-title">Page not found</h1>
      <p>
        That address does not match anything here. <a href={HOME_HREF}>Back to projects</a>
      </p>
    </>
  );
}
