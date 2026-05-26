import { tool } from "@opencode-ai/plugin"
import {
  buildArtifactsInspectArgs,
  buildArtifactsListArgs,
  buildBatchArgs,
  buildCrawlArgs,
  buildDoctorArgs,
  buildFetchArgs,
  buildMapArgs,
} from "../lib/aget_args"

type ToolContext = {
  worktree?: string
  directory?: string
}

const commandEnv = "AGET_OPENCODE_BIN"

function agetBinary(): string {
  return process.env[commandEnv] || "aget"
}

async function runAget(args: string[], context: ToolContext): Promise<string> {
  try {
    const proc = Bun.spawn([agetBinary(), "--envelope", "json", ...args], {
      cwd: context.worktree || context.directory || process.cwd(),
      stdout: "pipe",
      stderr: "pipe",
    })
    const [stdout, stderr, exitCode] = await Promise.all([
      new Response(proc.stdout).text(),
      new Response(proc.stderr).text(),
      proc.exited,
    ])

    const output = exitCode === 0 ? stdout.trim() : stderr.trim() || stdout.trim()
    if (output) {
      return output
    }

    return JSON.stringify({
      ok: false,
      schema_version: "aget.envelope.v1",
      command: "opencode",
      error: {
        code: "backend_unavailable",
        message: `aget exited with status ${exitCode} and produced no structured output`,
      },
    })
  } catch (error) {
    return JSON.stringify({
      ok: false,
      schema_version: "aget.envelope.v1",
      command: "opencode",
      error: {
        code: "backend_unavailable",
        message: `Unable to run aget. Install it on PATH or set ${commandEnv}: ${String(error)}`,
      },
    })
  }
}

function addOptional(args: string[], flag: string, value: string | number | undefined): void {
  if (value !== undefined) {
    args.push(flag, String(value))
  }
}

export const fetch = tool({
  description:
    "Fetch an HTTP(S) URL with the local aget CLI and its owned default extraction path. Returns aget's structured JSON envelope. Use explicit sessions only when the user has authorized access. If content looks gated and the site uses OAuth, ask before importing a real browser session; do not use automation login as the first path.",
  args: {
    url: tool.schema.string().describe("HTTP(S) URL to fetch."),
    sessions: tool.schema
      .array(tool.schema.string())
      .optional()
      .describe("Optional local aget session names to replay, in order."),
    content_format: tool.schema
      .enum(["markdown", "html", "text", "json"])
      .optional()
      .describe("Requested page content format. Defaults to markdown."),
    inline_content: tool.schema
      .enum(["auto", "always", "never"])
      .optional()
      .describe("Whether to include extracted content inline in the JSON envelope. Defaults to auto."),
    selector: tool.schema.string().optional().describe("Optional CSS selector to include."),
    exclude_selector: tool.schema
      .string()
      .optional()
      .describe("Optional CSS selector to exclude."),
    wait_for_selector: tool.schema
      .string()
      .optional()
      .describe("Optional CSS selector to wait for. JavaScript waits are rejected by aget."),
    max_chars: tool.schema
      .number()
      .int()
      .min(0)
      .optional()
      .describe("Optional deterministic content character limit."),
    timeout: tool.schema
      .number()
      .int()
      .positive()
      .optional()
      .describe("Optional timeout in seconds."),
    output: tool.schema
      .string()
      .optional()
      .describe("Optional local path for extracted content. The envelope still reports artifacts."),
    backend_options: tool.schema
      .array(tool.schema.string())
      .optional()
      .describe("Optional unstable backend key=value strings. Prefer first-class CLI flags when available; backend option namespace cleanup is tracked in the workpad."),
  },
  async execute(args, context) {
    return runAget(buildFetchArgs(args), context)
  },
})

export const batch = tool({
  description:
    "Fetch a finite explicit URL list with aget batch. Use this when the agent already has several known URLs to compare or save, not when it needs link discovery. Returns a structured JSON envelope with a manifest and per-item artifacts.",
  args: {
    urls: tool.schema.array(tool.schema.string()).optional().describe("Explicit URL inputs to fetch."),
    file: tool.schema.string().optional().describe("Optional newline-delimited URL file."),
    sessions: tool.schema.array(tool.schema.string()).optional().describe("Optional local aget session names to replay for each URL."),
    content_format: tool.schema.enum(["markdown", "html", "text", "json"]).optional().describe("Requested page content format for each item. Defaults to markdown."),
    selector: tool.schema.string().optional().describe("Optional CSS selector to include for each item."),
    exclude_selector: tool.schema.string().optional().describe("Optional CSS selector to exclude for each item."),
    wait_for_selector: tool.schema.string().optional().describe("Optional CSS selector to wait for before extraction."),
    max_chars: tool.schema.number().int().min(0).optional().describe("Optional deterministic character limit per item."),
    timeout: tool.schema.number().int().positive().optional().describe("Optional timeout in seconds."),
    concurrency: tool.schema.number().int().positive().optional().describe("Maximum number of URLs to fetch concurrently. Defaults to aget's CLI default."),
    output_dir: tool.schema.string().optional().describe("Optional local directory for the batch manifest and per-item artifacts."),
    fail_fast: tool.schema.boolean().optional().describe("Stop scheduling new URLs after the first failure."),
    backend_options: tool.schema.array(tool.schema.string()).optional().describe("Optional unstable backend key=value strings. Prefer first-class CLI flags when available."),
  },
  async execute(args, context) {
    return runAget(buildBatchArgs(args), context)
  },
})

export const map = tool({
  description:
    "Discover and filter links from one URL or one internal get artifact with aget map. Use this before crawl when the agent needs to inspect candidate links without fetching every discovered page.",
  args: {
    url: tool.schema.string().optional().describe("URL input to fetch and map."),
    artifact: tool.schema.string().optional().describe("Internal aget get run id to map instead of fetching a URL."),
    sessions: tool.schema.array(tool.schema.string()).optional().describe("Optional local aget session names to replay when fetching a URL input."),
    timeout: tool.schema.number().int().positive().optional().describe("Optional timeout in seconds."),
    selector: tool.schema.string().optional().describe("Optional CSS selector used before link extraction."),
    exclude_selector: tool.schema.string().optional().describe("Optional CSS selector to remove before link extraction."),
    wait_for_selector: tool.schema.string().optional().describe("Optional CSS selector to wait for before extraction."),
    any_origin: tool.schema.boolean().optional().describe("Allow links outside the source origin."),
    same_origin: tool.schema.boolean().optional().describe("Keep only links on the source origin; this is aget's default."),
    any_path: tool.schema.boolean().optional().describe("Allow links outside the source path prefix."),
    same_path: tool.schema.boolean().optional().describe("Keep only links under the source path prefix; this is aget's default."),
    include: tool.schema.array(tool.schema.string()).optional().describe("Simple include glob patterns. Repeatable."),
    exclude: tool.schema.array(tool.schema.string()).optional().describe("Simple exclude glob patterns. Repeatable."),
    max_links: tool.schema.number().int().positive().optional().describe("Maximum number of links to emit."),
    content_types: tool.schema.array(tool.schema.string()).optional().describe("Inferred content types to keep, such as text/html or application/pdf."),
    backend_options: tool.schema.array(tool.schema.string()).optional().describe("Optional unstable backend key=value strings. Prefer first-class CLI flags when available."),
  },
  async execute(args, context) {
    return runAget(buildMapArgs(args), context)
  },
})

export const crawl = tool({
  description:
    "Run a bounded same-origin/path aget crawl. Use only when the agent needs several pages from a section and has a clear limit; prefer map first when deciding scope. Returns a structured crawl manifest and per-page artifacts.",
  args: {
    url: tool.schema.string().describe("Start URL for bounded traversal."),
    limit: tool.schema.number().int().positive().describe("Maximum number of pages to fetch. Required by aget."),
    max_depth: tool.schema.number().int().min(0).optional().describe("Maximum link depth from the start URL."),
    timeout: tool.schema.number().int().positive().optional().describe("Optional timeout in seconds."),
    concurrency: tool.schema.number().int().positive().optional().describe("Maximum number of pages to fetch concurrently."),
    any_origin: tool.schema.boolean().optional().describe("Allow traversal outside the start origin."),
    same_origin: tool.schema.boolean().optional().describe("Keep traversal on the start origin; this is aget's default."),
    any_path: tool.schema.boolean().optional().describe("Allow traversal outside the start path prefix."),
    same_path: tool.schema.boolean().optional().describe("Keep traversal under the start path prefix; this is aget's default."),
    allow_domains: tool.schema.array(tool.schema.string()).optional().describe("Additional domains traversal may visit when same-origin is widened."),
    include: tool.schema.array(tool.schema.string()).optional().describe("Simple include glob patterns. Repeatable."),
    exclude: tool.schema.array(tool.schema.string()).optional().describe("Simple exclude glob patterns. Repeatable."),
    sessions: tool.schema.array(tool.schema.string()).optional().describe("Optional local aget session names to replay for each fetched page."),
    content_format: tool.schema.enum(["markdown", "html", "text", "json"]).optional().describe("Stored page artifact content format. Defaults to markdown."),
    selector: tool.schema.string().optional().describe("Optional CSS selector to include."),
    exclude_selector: tool.schema.string().optional().describe("Optional CSS selector to remove."),
    wait_for_selector: tool.schema.string().optional().describe("Optional CSS selector to wait for before extraction."),
    max_chars: tool.schema.number().int().min(0).optional().describe("Optional deterministic character limit per page."),
    output_dir: tool.schema.string().optional().describe("Optional local directory for crawl manifest and per-page artifacts."),
    backend_options: tool.schema.array(tool.schema.string()).optional().describe("Optional unstable backend key=value strings. Prefer first-class CLI flags when available."),
  },
  async execute(args, context) {
    return runAget(buildCrawlArgs(args), context)
  },
})

export const artifacts_list = tool({
  description:
    "List local aget run artifacts under AGET_HOME. Use this to find prior run IDs before inspecting or reusing local artifacts.",
  args: {},
  async execute(_args, context) {
    return runAget(buildArtifactsListArgs(), context)
  },
})

export const artifacts_inspect = tool({
  description:
    "Inspect one local aget run artifact by run ID. Use this before reading private or large artifact paths so the agent can choose the smallest needed file.",
  args: {
    run_id: tool.schema.string().describe("Local aget run ID to inspect."),
  },
  async execute(args, context) {
    return runAget(buildArtifactsInspectArgs(args.run_id), context)
  },
})

export const doctor = tool({
  description:
    "Run aget doctor readiness diagnostics. Use this when aget, Chrome/CDP, cmux, artifact storage, or OpenCode binary resolution appears misconfigured.",
  args: {
    quick: tool.schema.boolean().optional().describe("Skip slower probes such as run-artifact sizing."),
    checks: tool.schema.array(tool.schema.enum(["binary", "store", "artifacts", "chrome", "current-tab", "cmux", "opencode"])).optional().describe("Optional selected check groups."),
    timeout: tool.schema.number().int().positive().optional().describe("Optional timeout in seconds."),
  },
  async execute(args, context) {
    return runAget(buildDoctorArgs(args), context)
  },
})

export const session_list = tool({
  description:
    "List local aget session names through the aget CLI. Returns aget's structured JSON envelope.",
  args: {},
  async execute(_args, context) {
    return runAget(["session", "list"], context)
  },
})

export const session_inspect = tool({
  description:
    "Inspect one local aget session through the aget CLI. Secret cookie and storage values remain redacted.",
  args: {
    name: tool.schema.string().describe("Local aget session name to inspect."),
  },
  async execute(args, context) {
    return runAget(["session", "inspect", args.name], context)
  },
})

export const session_import_chrome = tool({
  description:
    "Import a scoped aget session from a user-approved real Chrome profile using aget's browser-neutral import surface and owned local Chrome/CDP path. Prefer this for OAuth-backed sites: the user logs in through their normal browser, then the agent imports only the allowed domains and verifies with fetch using the named session.",
  args: {
    profile: tool.schema
      .string()
      .describe("Chrome profile name or profile path approved by the user, e.g. Default."),
    name: tool.schema.string().describe("Local aget session name to create or replace."),
    domains: tool.schema
      .array(tool.schema.string())
      .min(1)
      .describe("Explicit allowed domains to import, e.g. ['example.com', 'www.example.com']."),
    timeout: tool.schema
      .number()
      .int()
      .positive()
      .optional()
      .describe("Optional timeout in seconds."),
  },
  async execute(args, context) {
    const cliArgs: string[] = []
    addOptional(cliArgs, "--timeout", args.timeout)
    cliArgs.push("session", "import", "browser", "--browser", "chrome", "--browser-profile", args.profile, "--name", args.name)
    for (const domain of args.domains) {
      cliArgs.push("--allow-domain", domain)
    }

    return runAget(cliArgs, context)
  },
})
