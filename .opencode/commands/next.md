---
description: Do the next workpad task
---

# Next: Do Next Task

Follow the workpads methodology to complete the next task.

## Step 1: Read State Files

Read these files first:

1. `AGENTS.md`
2. `project.md`
3. `WORKING.md`
4. `workpads/WORKPADS.md`
5. `workpads/{active-project}/tasks.md`
6. `workpads/{active-project}/knowledge.md`
7. `workpads/{active-project}/references.md`

## Step 2: Select A Task

Choose a pending `📋` task based on:

- Dependencies
- Current state
- Risk
- Testability
- Whether it unblocks source-of-truth cleanup, dependency removal, or product
  reliability

## Step 3: Execute

1. Mark the task `🚧 in_progress`.
2. Complete the task's acceptance criteria.
3. Update `references.md` with primary sources.
4. Update `knowledge.md` with decisions, findings, and open questions.
5. Assess confidence using `WORKING.md`.
6. Spawn focused review subagents when work is substantial, risky, confidence is below high, tests changed meaningfully, or a deliverable has settled.
7. Apply review feedback, record rejected feedback, or ask the user when direction is needed.
8. Mark the task `✅ completed`.
9. Add follow-up tasks if discovered.

## Rules

- Use `workpads/WORKPADS.md` to identify the active workpad; older research
  workpads are historical unless `WORKPADS.md` says otherwise.
- Do not commit without explicit user confirmation.
- If evidence is weak, record uncertainty instead of guessing.
- Do not mark tasks complete while material review findings remain unresolved.

Start now.
