import type { ReactNode } from "react";
import { HOME_HREF } from "../router";

function focusMain(): void {
  document.getElementById("main")?.focus();
}

export function Layout({ children }: { children: ReactNode }) {
  return (
    <>
      <button type="button" className="skip-link" onClick={focusMain}>
        Skip to content
      </button>
      <header className="site-header">
        <div className="shell">
          <a className="brand" href={HOME_HREF}>
            Shipcheck
          </a>
          <nav aria-label="Main">
            <a href={HOME_HREF}>Projects</a>
          </nav>
        </div>
      </header>
      <div className="shell">
        <main id="main" tabIndex={-1}>
          {children}
        </main>
        <footer className="site-footer">
          Scores are computed by the server from the stored findings.
        </footer>
      </div>
    </>
  );
}
