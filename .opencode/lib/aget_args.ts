function addOptional(args: string[], flag: string, value: string | number | undefined): void {
  if (value !== undefined) {
    args.push(flag, String(value))
  }
}

function addRepeated(args: string[], flag: string, values: string[] | undefined): void {
  for (const value of values || []) {
    args.push(flag, value)
  }
}

function addFlag(args: string[], flag: string, enabled: boolean | undefined): void {
  if (enabled) {
    args.push(flag)
  }
}

type ExtractionArgs = {
  sessions?: string[]
  content_format?: "markdown" | "html" | "text" | "json"
  selector?: string
  exclude_selector?: string
  wait_for_selector?: string
  max_chars?: number
  backend_options?: string[]
}

function addExtractionArgs(args: string[], input: ExtractionArgs): void {
  addRepeated(args, "--session", input.sessions)
  if (input.content_format) {
    args.push("--content-format", input.content_format)
  }
  addOptional(args, "--selector", input.selector)
  addOptional(args, "--exclude-selector", input.exclude_selector)
  addOptional(args, "--wait-for-selector", input.wait_for_selector)
  addOptional(args, "--max-chars", input.max_chars)
  addRepeated(args, "--backend-option", input.backend_options)
}

export type FetchArgs = ExtractionArgs & {
  url: string
  inline_content?: "auto" | "always" | "never"
  timeout?: number
  output?: string
}

export function buildFetchArgs(input: FetchArgs): string[] {
  const cliArgs: string[] = []
  addOptional(cliArgs, "--timeout", input.timeout)
  cliArgs.push("get", input.url)
  addExtractionArgs(cliArgs, {
    ...input,
    content_format: input.content_format || "markdown",
  })
  addOptional(cliArgs, "--inline-content", input.inline_content)
  addOptional(cliArgs, "--output", input.output)
  return cliArgs
}

export type BatchArgs = ExtractionArgs & {
  urls?: string[]
  file?: string
  timeout?: number
  concurrency?: number
  output_dir?: string
  fail_fast?: boolean
}

export function buildBatchArgs(input: BatchArgs): string[] {
  const cliArgs: string[] = []
  addOptional(cliArgs, "--timeout", input.timeout)
  cliArgs.push("batch")
  for (const url of input.urls || []) {
    cliArgs.push(url)
  }
  addOptional(cliArgs, "--file", input.file)
  addExtractionArgs(cliArgs, input)
  addOptional(cliArgs, "--concurrency", input.concurrency)
  addOptional(cliArgs, "--output-dir", input.output_dir)
  addFlag(cliArgs, "--fail-fast", input.fail_fast)
  return cliArgs
}

export type MapArgs = {
  url?: string
  artifact?: string
  sessions?: string[]
  timeout?: number
  selector?: string
  exclude_selector?: string
  wait_for_selector?: string
  any_origin?: boolean
  same_origin?: boolean
  any_path?: boolean
  same_path?: boolean
  include?: string[]
  exclude?: string[]
  max_links?: number
  content_types?: string[]
  backend_options?: string[]
}

export function buildMapArgs(input: MapArgs): string[] {
  const cliArgs: string[] = []
  addOptional(cliArgs, "--timeout", input.timeout)
  cliArgs.push("map")
  if (input.url) {
    cliArgs.push(input.url)
  }
  addOptional(cliArgs, "--artifact", input.artifact)
  addRepeated(cliArgs, "--session", input.sessions)
  addOptional(cliArgs, "--selector", input.selector)
  addOptional(cliArgs, "--exclude-selector", input.exclude_selector)
  addOptional(cliArgs, "--wait-for-selector", input.wait_for_selector)
  addFlag(cliArgs, "--any-origin", input.any_origin)
  addFlag(cliArgs, "--same-origin", input.same_origin)
  addFlag(cliArgs, "--any-path", input.any_path)
  addFlag(cliArgs, "--same-path", input.same_path)
  addRepeated(cliArgs, "--include", input.include)
  addRepeated(cliArgs, "--exclude", input.exclude)
  addOptional(cliArgs, "--max-links", input.max_links)
  addRepeated(cliArgs, "--content-type", input.content_types)
  addRepeated(cliArgs, "--backend-option", input.backend_options)
  return cliArgs
}

export type CrawlArgs = ExtractionArgs & {
  url: string
  limit: number
  max_depth?: number
  timeout?: number
  concurrency?: number
  any_origin?: boolean
  same_origin?: boolean
  any_path?: boolean
  same_path?: boolean
  allow_domains?: string[]
  include?: string[]
  exclude?: string[]
  output_dir?: string
}

export function buildCrawlArgs(input: CrawlArgs): string[] {
  const cliArgs: string[] = []
  addOptional(cliArgs, "--timeout", input.timeout)
  cliArgs.push("crawl", input.url, "--limit", String(input.limit))
  addOptional(cliArgs, "--max-depth", input.max_depth)
  addOptional(cliArgs, "--concurrency", input.concurrency)
  addFlag(cliArgs, "--any-origin", input.any_origin)
  addFlag(cliArgs, "--same-origin", input.same_origin)
  addFlag(cliArgs, "--any-path", input.any_path)
  addFlag(cliArgs, "--same-path", input.same_path)
  addRepeated(cliArgs, "--allow-domain", input.allow_domains)
  addRepeated(cliArgs, "--include", input.include)
  addRepeated(cliArgs, "--exclude", input.exclude)
  addExtractionArgs(cliArgs, input)
  addOptional(cliArgs, "--output-dir", input.output_dir)
  return cliArgs
}

export function buildArtifactsListArgs(): string[] {
  return ["artifacts", "list"]
}

export function buildArtifactsInspectArgs(runId: string): string[] {
  return ["artifacts", "inspect", runId]
}

export type DoctorArgs = {
  quick?: boolean
  checks?: Array<"binary" | "store" | "artifacts" | "chrome" | "current-tab" | "cmux" | "opencode">
  timeout?: number
}

export function buildDoctorArgs(input: DoctorArgs): string[] {
  const cliArgs: string[] = []
  addOptional(cliArgs, "--timeout", input.timeout)
  cliArgs.push("doctor")
  addFlag(cliArgs, "--quick", input.quick)
  addRepeated(cliArgs, "--check", input.checks)
  return cliArgs
}
