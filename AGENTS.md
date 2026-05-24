# AGENTS.md

This repository is for `aget`, a local-first, auth-aware agent web context tool.

## Source Of Truth

- `project.md` describes the original product goal and feature inventory.
- `WORKING.md` describes project-wide working practices, confidence assessment, and review-subagent loops.
- `workpads/WORKPADS.md` defines current focus and what to load.
- `workpads/{project}/tasks.md` tracks executable work.
- `workpads/{project}/knowledge.md` records decisions and lessons.
- `workpads/{project}/references.md` records external research.

Progress persists in files and git, not conversation context.

## Current Phase

The project is in **post-migration CLI productization**. The active product
surface is the Rust CLI and its structured envelope. Do not add an MCP server or
MCP dependency unless the user explicitly reverses this direction.

## Mandatory Workflow

Before doing task work:

1. Read `workpads/WORKPADS.md`.
2. Read `project.md` for product intent.
3. Read `WORKING.md` for working practices and review expectations.
4. Read the active workpad's `tasks.md`, `knowledge.md`, and `references.md` if they exist.
5. Select a pending task based on dependencies, risk, and testability.
6. Mark the task `🚧 in_progress` before working.
7. Record findings in `knowledge.md` or `references.md`.
8. Assess confidence and use review subagents as described in `WORKING.md` when work is substantial, risky, or settling.
9. Mark the task `✅ completed` only after acceptance criteria and required review follow-up are handled.

## Code Documentation

Use succinct comments when repository-specific terms or boundaries are not self-evident. Comments are especially helpful around terms like `backend`, `extractor`, `session`, `profile`, `artifact`, and `envelope`, where the code may otherwise hide whether it is talking about local processes, browser state, persisted data, or the public API.

## Git Rules

- Do not commit or push without explicit user confirmation.
- If asked to commit, show the files and message first.
- Never use destructive git commands unless explicitly requested.

## Research Rules

- Prefer primary sources: upstream repos, docs, licenses, API references.
- Record license implications when evaluating reusable code.
- For external projects, distinguish reusable libraries from hosted-service-only code.
- For auth/session behavior, explicitly note privacy and local-data risks.

## Implementation Direction

The implementation language is Rust. Future implementation work should preserve
the CLI as the behavior source of truth and keep integrations as thin
CLI/envelope callers unless a workpad task explicitly approves a different
boundary.

## Safety Boundary

`aget` should only help users process content they are authorized to access. It must not be designed to bypass paywalls, access controls, or site policies. Authenticated data must stay local by default.

`aget` is a generic fetcher. Do not add site-specific paywall/login detection, built-in site names, or site-shaped retry advice to the binary. Site-specific reasoning belongs to the calling agent or an agent-facing skill that interprets generic extraction outcomes.

## Verification

During research, verification means:

- Findings cite source URLs or local reference paths.
- Claims about licenses or architecture are backed by primary sources.
- Open questions are recorded explicitly instead of guessed.

During implementation, verification requirements will be added once stack and test strategy are chosen.
During implementation, verify touched behavior with focused tests plus the
standard formatting/test gate recorded in the active workpad. For documentation
work, verify command examples against current CLI help and grep for stale
dependency claims.
