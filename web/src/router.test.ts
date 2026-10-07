import { describe, expect, it } from "vitest";
import { HOME_HREF, parseRoute, projectHref, scanHref, titleFor } from "./router";

describe("parseRoute", () => {
  it("treats empty hashes as the project list", () => {
    for (const hash of ["", "#", "#/", HOME_HREF]) {
      expect(parseRoute(hash)).toEqual({ page: "projects" });
    }
  });

  it("decodes project names that contain slashes", () => {
    expect(parseRoute("#/p/eyadstein%2Fshipcheck")).toEqual({
      page: "project",
      project: "eyadstein/shipcheck",
    });
  });

  it("parses scan ids", () => {
    expect(parseRoute("#/scan/abc-123")).toEqual({ page: "scan", id: "abc-123" });
  });

  it("reports unknown, incomplete and malformed addresses as missing", () => {
    for (const hash of ["#/nope/x", "#/p", "#/scan/a/b", "#/p/%E0%A4%A"]) {
      expect(parseRoute(hash)).toEqual({ page: "missing" });
    }
  });
});

describe("links", () => {
  it("round trips through the parser", () => {
    expect(parseRoute(projectHref("a/b c"))).toEqual({ page: "project", project: "a/b c" });
    expect(parseRoute(scanHref("id 1"))).toEqual({ page: "scan", id: "id 1" });
  });

  it("builds titles", () => {
    expect(titleFor({ page: "projects" })).toBe("Projects | Shipcheck");
    expect(titleFor({ page: "project", project: "a/b" })).toBe("a/b | Shipcheck");
    expect(titleFor({ page: "missing" })).toBe("Not found | Shipcheck");
  });
});
