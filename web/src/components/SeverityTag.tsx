import type { Severity } from "../types";

export function SeverityTag({ severity }: { severity: Severity }) {
  return <span className={`tag tag-${severity}`}>{severity}</span>;
}
