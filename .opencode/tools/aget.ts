import { tool } from "@opencode-ai/plugin"

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
      .describe("Optional unstable backend.key=value strings; the owned extractor currently supports crawl4ai.target_elements, crawl4ai.excluded_tags, and crawl4ai.delay_before_return_html."),
  },
  async execute(args, context) {
    const cliArgs: string[] = []
    addOptional(cliArgs, "--timeout", args.timeout)
    cliArgs.push("get", args.url)

    for (const session of args.sessions || []) {
      cliArgs.push("--session", session)
    }
    cliArgs.push("--content-format", args.content_format || "markdown")
    addOptional(cliArgs, "--inline-content", args.inline_content)
    addOptional(cliArgs, "--selector", args.selector)
    addOptional(cliArgs, "--exclude-selector", args.exclude_selector)
    addOptional(cliArgs, "--wait-for-selector", args.wait_for_selector)
    addOptional(cliArgs, "--max-chars", args.max_chars)
    addOptional(cliArgs, "--output", args.output)
    for (const option of args.backend_options || []) {
      cliArgs.push("--backend-option", option)
    }

    return runAget(cliArgs, context)
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
    "Import a scoped aget session from a user-approved real Chrome profile using aget's owned local Chrome/CDP path. Prefer this for OAuth-backed sites: the user logs in through their normal browser, then the agent imports only the allowed domains and verifies with fetch using the named session.",
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
    cliArgs.push("session", "import", "chrome", "--chrome-profile", args.profile, "--name", args.name)
    for (const domain of args.domains) {
      cliArgs.push("--allow-domain", domain)
    }

    return runAget(cliArgs, context)
  },
})
