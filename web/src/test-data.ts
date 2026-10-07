import type { Finding } from "./types";

export function makeFinding(overrides: Partial<Finding> = {}): Finding {
  return {
    ruleId: "SEC-101",
    category: "security",
    severity: "high",
    message: "User input reaches a SQL query",
    file: "src/db.js",
    line: 10,
    fix: null,
    ...overrides,
  };
}
