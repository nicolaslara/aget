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
    const proc = Bun.spawn([agetBinary(), "--json", ...args], {
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
      command: "opencode",
      error: {
        code: "backend_unavailable",
        message: `aget exited with status ${exitCode} and produced no structured output`,
      },
    })
  } catch (error) {
    return JSON.stringify({
      ok: false,
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
    "Fetch an HTTP(S) URL with the local aget CLI. Returns aget's structured JSON envelope. Use sessions only when the user has authorized access to the target content.",
  args: {
    url: tool.schema.string().describe("HTTP(S) URL to fetch."),
    sessions: tool.schema
      .array(tool.schema.string())
      .optional()
      .describe("Optional local aget session names to replay, in order."),
    format: tool.schema
      .enum(["markdown", "html", "text", "json"])
      .optional()
      .describe("Requested page content format. Defaults to markdown."),
    selector: tool.schema.string().optional().describe("Optional CSS selector to include."),
    exclude_selector: tool.schema
      .string()
      .optional()
      .describe("Optional CSS selector to exclude."),
    wait_for: tool.schema
      .string()
      .optional()
      .describe("Optional CSS wait condition. JavaScript waits are rejected by aget."),
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
    out: tool.schema
      .string()
      .optional()
      .describe("Optional local path for extracted content. The envelope still reports artifacts."),
    extractor_options: tool.schema
      .array(tool.schema.string())
      .optional()
      .describe("Optional aget extractor options as backend.key=value strings."),
  },
  async execute(args, context) {
    const cliArgs: string[] = []
    addOptional(cliArgs, "--timeout", args.timeout)
    cliArgs.push("get", args.url)

    for (const session of args.sessions || []) {
      cliArgs.push("--session", session)
    }
    cliArgs.push("--format", args.format || "markdown")
    addOptional(cliArgs, "--selector", args.selector)
    addOptional(cliArgs, "--exclude-selector", args.exclude_selector)
    addOptional(cliArgs, "--wait-for", args.wait_for)
    addOptional(cliArgs, "--max-chars", args.max_chars)
    addOptional(cliArgs, "--out", args.out)
    for (const option of args.extractor_options || []) {
      cliArgs.push("--extractor-option", option)
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
