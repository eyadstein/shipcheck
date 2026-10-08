import { appendFileSync, readFileSync } from "node:fs";
import { uploadReport, upsertComment } from "./github.ts";
import type { FetchLike } from "./github.ts";
import { parseReport } from "./parse.ts";
import {
  DEFAULT_MAX_FINDINGS,
  MARKER,
  SUMMARY_MAX_FINDINGS,
  normalizeScanPath,
  parseFlag,
  parseThreshold,
  renderComment,
  scoreTone,
} from "./report.ts";
import type { RenderOptions } from "./report.ts";
import type { Report } from "./types.ts";

const http: FetchLike = fetch;

interface PullRequest {
  number: number;
  headSha: string | null;
}

function env(name: string): string {
  return process.env[name]?.trim() ?? "";
}

function describeError(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

/** Prints a workflow command. Newlines and percent signs must be escaped. */
function annotate(level: "warning" | "error", message: string): void {
  const safe = message.replace(/%/g, "%25").replace(/\r/g, "%0D").replace(/\n/g, "%0A");
  console.log(`::${level}::${safe}`);
}

function setOutput(name: string, value: string): void {
  const file = env("GITHUB_OUTPUT");
  if (file !== "") {
    appendFileSync(file, `${name}=${value}\n`);
  }
}

function field(value: unknown, key: string): unknown {
  return typeof value === "object" && value !== null
    ? (value as Record<string, unknown>)[key]
    : undefined;
}

function readPullRequest(): PullRequest | null {
  const path = env("GITHUB_EVENT_PATH");
  if (path === "") {
    return null;
  }
  try {
    const event: unknown = JSON.parse(readFileSync(path, "utf8"));
    const pr = field(event, "pull_request");
    const number = field(pr, "number");
    const sha = field(field(pr, "head"), "sha");
    if (typeof number !== "number") {
      return null;
    }
    return { number, headSha: typeof sha === "string" ? sha : null };
  } catch {
    return null;
  }
}

async function postComment(report: Report, options: RenderOptions, pr: PullRequest): Promise<void> {
  const token = env("SHIPCHECK_TOKEN");
  const repository = options.repository;
  if (token === "" || repository === null) {
    throw new Error("a github-token and GITHUB_REPOSITORY are required");
  }
  const body = renderComment(report, { ...options, maxFindings: DEFAULT_MAX_FINDINGS });
  const target = {
    apiUrl: env("GITHUB_API_URL") || "https://api.github.com",
    repository,
    issueNumber: pr.number,
    token,
  };
  const result = await upsertComment(http, target, MARKER, body);
  console.log(`Pull request comment ${result}.`);
}

async function upload(report: Report, options: RenderOptions): Promise<void> {
  const apiUrl = env("SHIPCHECK_API_URL");
  if (apiUrl === "") {
    return;
  }
  const apiKey = env("SHIPCHECK_API_KEY");
  if (apiKey === "") {
    throw new Error("api-url is set but api-key is empty");
  }
  const target = {
    apiUrl,
    apiKey,
    project: env("SHIPCHECK_PROJECT") || options.repository || "unknown",
    gitRef: env("GITHUB_REF_NAME") || null,
  };
  await uploadReport(http, target, report);
  console.log("Report uploaded to the Shipcheck API.");
}

async function main(): Promise<number> {
  const reportPath = env("SHIPCHECK_REPORT");
  if (reportPath === "") {
    throw new Error("SHIPCHECK_REPORT is not set");
  }
  const failUnder = parseThreshold(env("SHIPCHECK_FAIL_UNDER"));
  const wantComment = parseFlag(env("SHIPCHECK_COMMENT"), true);
  const report = parseReport(readFileSync(reportPath, "utf8"));
  setOutput("score", String(report.score));
  setOutput("findings", String(report.findings.length));

  const pr = readPullRequest();
  const options: RenderOptions = {
    failUnder,
    scanPath: normalizeScanPath(env("SHIPCHECK_SCAN_PATH") || "."),
    repository: env("GITHUB_REPOSITORY") || null,
    sha: pr?.headSha ?? (env("GITHUB_SHA") || null),
    serverUrl: env("GITHUB_SERVER_URL") || "https://github.com",
  };

  const summaryFile = env("GITHUB_STEP_SUMMARY");
  if (summaryFile !== "") {
    const summary = renderComment(report, { ...options, maxFindings: SUMMARY_MAX_FINDINGS });
    appendFileSync(summaryFile, `${summary}\n`);
  }

  if (wantComment && pr !== null) {
    try {
      await postComment(report, options, pr);
    } catch (error) {
      annotate(
        "warning",
        `Could not post the pull request comment: ${describeError(error)}. The report is in the job summary.`,
      );
    }
  }

  try {
    await upload(report, options);
  } catch (error) {
    annotate("warning", `Could not upload the report: ${describeError(error)}`);
  }

  if (failUnder > 0 && report.score < failUnder) {
    annotate("error", `Shipcheck score ${report.score} is below the required ${failUnder}.`);
    return 1;
  }
  console.log(`Shipcheck score ${report.score}/100 (${scoreTone(report.score)}).`);
  return 0;
}

main().then(
  (code) => {
    process.exitCode = code;
  },
  (error: unknown) => {
    annotate("error", describeError(error));
    process.exitCode = 1;
  },
);
