import { expect, test } from "bun:test"
import {
  buildArtifactsInspectArgs,
  buildArtifactsListArgs,
  buildBatchArgs,
  buildCrawlArgs,
  buildDoctorArgs,
  buildExtractArgs,
  buildMapArgs,
  buildSearchPageArgs,
} from "../lib/aget_args"

test("builds batch args with repeated sessions and output directory", () => {
  expect(
    buildBatchArgs({
      urls: ["https://example.com/a", "https://example.com/b"],
      sessions: ["docs"],
      content_format: "markdown",
      fresh: true,
      cache_ttl: 60,
      concurrency: 3,
      output_dir: "/tmp/aget-batch",
      fail_fast: true,
    }),
  ).toEqual([
    "batch",
    "https://example.com/a",
    "https://example.com/b",
    "--session",
    "docs",
    "--content-format",
    "markdown",
    "--fresh",
    "--cache-ttl",
    "60",
    "--concurrency",
    "3",
    "--output-dir",
    "/tmp/aget-batch",
    "--fail-fast",
  ])
})

test("builds map args for artifact-first link discovery", () => {
  expect(
    buildMapArgs({
      artifact: "run-123",
      any_path: true,
      include: ["*/docs/*"],
      content_types: ["text/html"],
      cache_policy: "off",
      max_links: 50,
    }),
  ).toEqual([
    "map",
    "--artifact",
    "run-123",
    "--any-path",
    "--include",
    "*/docs/*",
    "--max-links",
    "50",
    "--cache-policy",
    "off",
    "--content-type",
    "text/html",
  ])
})

test("builds crawl args with required limit and traversal bounds", () => {
  expect(
    buildCrawlArgs({
      url: "https://example.com/docs/",
      limit: 10,
      max_depth: 2,
      concurrency: 2,
      allow_domains: ["cdn.example.com"],
      sessions: ["docs"],
      content_format: "html",
      cache_policy: "refresh",
      output_dir: "/tmp/aget-crawl",
    }),
  ).toEqual([
    "crawl",
    "https://example.com/docs/",
    "--limit",
    "10",
    "--max-depth",
    "2",
    "--concurrency",
    "2",
    "--allow-domain",
    "cdn.example.com",
    "--session",
    "docs",
    "--content-format",
    "html",
    "--cache-policy",
    "refresh",
    "--output-dir",
    "/tmp/aget-crawl",
  ])
})

test("builds artifact and doctor args", () => {
  expect(buildArtifactsListArgs()).toEqual(["artifacts", "list"])
  expect(buildArtifactsInspectArgs("run-456")).toEqual(["artifacts", "inspect", "run-456"])
  expect(
    buildSearchPageArgs({
      artifact: "run-789",
      query: "install instructions",
      max_results: 4,
      allow_private_content: true,
    }),
  ).toEqual([
    "search-page",
    "--artifact",
    "run-789",
    "--query",
    "install instructions",
    "--max-results",
    "4",
    "--allow-private-content",
  ])
  expect(
    buildExtractArgs({
      manifest: "/tmp/crawl/manifest.json",
      schema: "/tmp/schema.json",
      fields: ["tables", "links"],
      allow_private_content: true,
    }),
  ).toEqual([
    "extract",
    "--manifest",
    "/tmp/crawl/manifest.json",
    "--schema",
    "/tmp/schema.json",
    "--field",
    "tables",
    "--field",
    "links",
    "--allow-private-content",
  ])
  expect(buildDoctorArgs({ quick: true, checks: ["binary", "opencode"] })).toEqual([
    "doctor",
    "--quick",
    "--check",
    "binary",
    "--check",
    "opencode",
  ])
})
