export const CATEGORIES = ["legal", "security", "design"] as const;
export type Category = (typeof CATEGORIES)[number];

export const SEVERITIES = ["info", "low", "medium", "high", "critical"] as const;
export type Severity = (typeof SEVERITIES)[number];

export interface Finding {
  ruleId: string;
  category: Category;
  severity: Severity;
  message: string;
  file: string;
  line: number;
  fix: string | null;
}

export interface Report {
  score: number;
  findings: Finding[];
}
