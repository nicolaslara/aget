---
applyTo: "**"
---

# aget

Use the installed `aget` CLI when a task needs agent-ready web context, local
browser/current-tab extraction, bounded discovery, structured extraction, or
reuse of previous `aget` artifacts.

- Prefer `aget --envelope json ...` when the result feeds an agent workflow.
- Use `aget get` for one known URL, `aget batch` for several known URLs,
  `aget map` for one-page link discovery, and `aget crawl <url> --limit <n>`
  for bounded traversal.
- Use `aget search-page --artifact <run-id>` and `aget extract --artifact
  <run-id>` for artifact-first narrowing and structured extraction.
- Use `aget interact` only with a local action-plan file and explicit user
  approval for actions, private content, sensitive input, submit, or screenshot
  capture.
- Do not collect credentials or silently use ambient browser auth. Authenticated
  content requires explicit user approval and named local sessions.
- Treat session files, screenshots, current-tab output, and authenticated
  artifacts as private local data.
