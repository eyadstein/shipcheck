import { describe, expect, it } from "vitest";
import { uploadReport, upsertComment } from "./github.ts";
import type { CommentTarget, FetchLike } from "./github.ts";
import { MARKER } from "./report.ts";
import type { Report } from "./types.ts";

interface Call {
  url: string;
  method: string;
  headers: Record<string, string> | undefined;
  body: unknown;
}

interface Reply {
  status: number;
  body?: unknown;
}

function makeHttp(handler: (call: Call) => Reply): { http: FetchLike; calls: Call[] } {
  const calls: Call[] = [];
  const http: FetchLike = async (url, init) => {
    const call: Call = {
      url,
      method: init?.method ?? "GET",
      headers: init?.headers,
      body: init?.body === undefined ? undefined : JSON.parse(init.body),
    };
    calls.push(call);
    const reply = handler(call);
    return {
      ok: reply.status >= 200 && reply.status < 300,
      status: reply.status,
      json: async () => reply.body,
    };
  };
  return { http, calls };
}

const TARGET: CommentTarget = {
  apiUrl: "https://api.github.com",
  repository: "o/r",
  issueNumber: 7,
  token: "secret",
};

describe("upsertComment", () => {
  it("creates a comment when there is none", async () => {
    const { http, calls } = makeHttp((call) =>
      call.method === "GET" ? { status: 200, body: [] } : { status: 201, body: {} },
    );
    expect(await upsertComment(http, TARGET, MARKER, "hello")).toBe("created");
    expect(calls.map((call) => call.method)).toEqual(["GET", "POST"]);
    expect(calls[1]?.url).toBe("https://api.github.com/repos/o/r/issues/7/comments");
    expect(calls[1]?.body).toEqual({ body: "hello" });
  });

  it("updates the comment that carries the marker", async () => {
    const { http, calls } = makeHttp((call) =>
      call.method === "GET"
        ? {
            status: 200,
            body: [
              { id: 5, body: "unrelated" },
              { id: 9, body: `${MARKER}\nold report` },
            ],
          }
        : { status: 200, body: {} },
    );
    expect(await upsertComment(http, TARGET, MARKER, "new")).toBe("updated");
    expect(calls.map((call) => call.method)).toEqual(["GET", "PATCH"]);
    expect(calls[1]?.url).toBe("https://api.github.com/repos/o/r/issues/comments/9");
  });

  it("creates a new comment when the old one cannot be edited", async () => {
    const { http, calls } = makeHttp((call) => {
      if (call.method === "GET") return { status: 200, body: [{ id: 9, body: MARKER }] };
      if (call.method === "PATCH") return { status: 403 };
      return { status: 201, body: {} };
    });
    expect(await upsertComment(http, TARGET, MARKER, "new")).toBe("created");
    expect(calls.map((call) => call.method)).toEqual(["GET", "PATCH", "POST"]);
  });

  it("reports a failure to list comments", async () => {
    const { http } = makeHttp(() => ({ status: 500 }));
    await expect(upsertComment(http, TARGET, MARKER, "x")).rejects.toThrow(/list comments/);
  });

  it("looks through several pages of comments", async () => {
    const firstPage = Array.from({ length: 100 }, (_, index) => ({ id: index + 1, body: "x" }));
    const { http, calls } = makeHttp((call) => {
      if (call.method === "PATCH") return { status: 200, body: {} };
      return call.url.endsWith("page=1")
        ? { status: 200, body: firstPage }
        : { status: 200, body: [{ id: 500, body: MARKER }] };
    });
    expect(await upsertComment(http, TARGET, MARKER, "x")).toBe("updated");
    expect(calls.map((call) => call.method)).toEqual(["GET", "GET", "PATCH"]);
    expect(calls[2]?.url.endsWith("/comments/500")).toBe(true);
  });

  it("sends the token as a bearer credential", async () => {
    const { http, calls } = makeHttp((call) =>
      call.method === "GET" ? { status: 200, body: [] } : { status: 201, body: {} },
    );
    await upsertComment(http, TARGET, MARKER, "x");
    expect(calls[0]?.headers?.["authorization"]).toBe("Bearer secret");
  });
});

describe("uploadReport", () => {
  const report: Report = {
    score: 90,
    findings: [
      {
        ruleId: "SEC-101",
        category: "security",
        severity: "high",
        message: "m",
        file: "a.js",
        line: 3,
        fix: null,
      },
    ],
  };

  it("posts the findings in the API wire format", async () => {
    const { http, calls } = makeHttp(() => ({ status: 201, body: {} }));
    await uploadReport(
      http,
      { apiUrl: "https://api.example/", apiKey: "k", project: "o/r", gitRef: "main" },
      report,
    );
    expect(calls[0]?.url).toBe("https://api.example/api/v1/scans");
    expect(calls[0]?.headers?.["x-api-key"]).toBe("k");
    const body = calls[0]?.body as { project: string; git_ref: string; findings: Array<{ rule_id: string }> };
    expect(body.project).toBe("o/r");
    expect(body.git_ref).toBe("main");
    expect(body.findings[0]?.rule_id).toBe("SEC-101");
  });

  it("fails when the API refuses the report", async () => {
    const { http } = makeHttp(() => ({ status: 401 }));
    await expect(
      uploadReport(http, { apiUrl: "https://api.example", apiKey: "bad", project: "o/r", gitRef: null }, report),
    ).rejects.toThrow(/status 401/);
  });
});
