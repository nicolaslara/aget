# Working Practices

This is the living project-wide agreement for how agents should work on `aget`. It complements `project.md` and the active workpads. Update it when the workflow changes.

## Purpose

`aget` should be built with clean, maintainable code and explicit review loops. Agents should not only execute tasks; they should periodically evaluate confidence, invite independent critique, and either improve the work or ask the user for direction.

## Core Loop

Use this loop for `/next` and similar task execution:

1. Load `AGENTS.md`, `project.md`, `WORKING.md`, and the active workpad files.
2. Select a task based on dependencies, risk, and testability.
3. Mark the task `🚧 in_progress` before doing work.
4. Complete the acceptance criteria with the smallest correct change.
5. Verify the work using the task's evidence standard.
6. Record findings, decisions, references, and open questions in the relevant workpad or project-level docs.
7. Assess confidence and decide whether a review subagent is needed.
8. Incorporate review feedback, document rejected feedback, or ask the user when the decision is product-sensitive.
9. Mark the task `✅ completed` only after acceptance criteria and review requirements are satisfied.

## Confidence Assessment

At natural settling points, the executing agent should state or record a confidence level for the completed work.

| Level | Meaning | Expected Action |
| --- | --- | --- |
| High | Evidence is strong; implementation or research is narrow and verified. | Proceed, but still use periodic review for important deliverables. |
| Medium | Work is likely correct but has assumptions, weak evidence, or meaningful scope. | Prefer a focused review subagent before completion. |
| Low | Work depends on unclear requirements, fragile code, weak tests, or uncertain sources. | Ask for review or user direction before treating it as complete. |

Confidence should consider:

- Whether acceptance criteria were directly satisfied.
- Whether tests or research evidence cover the real behavior we care about.
- Whether code changes are cohesive and maintainable.
- Whether security/privacy implications are understood.
- Whether there are unresolved product decisions.

## Review Subagents

Spawn review subagents periodically as work settles, especially after substantial implementation, test changes, architecture decisions, or security-sensitive changes.

Preferred reviewers:

- Latest GPT-family model for pragmatic engineering, test adequacy, and implementation review.
- Claude Opus-family model for architecture, maintainability, product coherence, and security/privacy critique.

Review prompts should be specific. Ask the reviewer to inspect a deliverable from one perspective and return concrete findings, confidence, and recommended actions.

Useful review types:

- Test adequacy: Do new or changed tests actually verify the intended behavior? Were existing tests weakened, deleted, or turned into pass-throughs?
- Feature coverage: Does the implementation cover the promised user-facing deliverable and edge cases?
- Code quality: Is the change minimal, idiomatic, cohesive, and easy to maintain?
- Architecture cohesion: Does the change fit the project direction, boundaries, and vocabulary?
- Security/privacy: Could credentials, authenticated content, logs, caches, screenshots, or provenance leak unexpectedly?
- Research quality: Are claims backed by primary sources, and are licenses/platform constraints represented accurately?

## Acting On Feedback

After subagent review:

- Fix issues that are clearly correct and within the current task.
- Record accepted design feedback in the relevant `knowledge.md` or project-level doc.
- Record rejected feedback when the rejection affects future work.
- Ask the user when feedback requires a product decision, wider scope, tradeoff, or change to the agreed direction.
- Do not mark a task complete while material review findings remain unresolved.

## Test Integrity

When tests are added or changed:

- Preserve the behavioral intent of existing tests unless the task explicitly changes that behavior.
- Do not replace meaningful assertions with superficial checks.
- Prefer tests that fail for the bug or missing feature before the fix.
- Review whether mocks and fixtures still exercise the real integration boundary we care about.
- For substantial test edits, request a test adequacy review before completion.

## Project-Level Knowledge

Use workpad files for active project execution. Use project-level docs for practices and decisions that apply across workpads.

Current project-level docs:

- `project.md`: product goal, feature inventory, priorities, and global tasks.
- `WORKING.md`: living working practices and review loop.

Add new project-level docs only when a topic applies across workpads, such as security model, architecture principles, glossary, release process, or testing strategy.

## Research Phase Notes

During research, review subagents should focus on source quality, missed comparable projects, licensing implications, privacy risks, and whether recommendations follow from evidence.

During implementation, review subagents should additionally inspect code, tests, maintainability, and behavior against the selected architecture.
