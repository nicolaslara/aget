# Workpads

Project-specific working documentation for AI assistants.

See [`WORKPADS.md`](./WORKPADS.md) for current focus and active projects.

## Structure

```text
workpads/
├── WORKPADS.md
├── {project}/
│   ├── knowledge.md
│   ├── references.md
│   └── tasks.md
└── README.md
```

## Standard Files

| File | Purpose |
| --- | --- |
| `WORKPADS.md` | Active projects and current focus. |
| `knowledge.md` | Design decisions, specs, lessons learned. |
| `references.md` | External references with quality notes. |
| `tasks.md` | Current task list with acceptance criteria. |

## Task States

```text
📋 pending      - Not yet started
🚧 in_progress  - Currently working on
✅ completed    - Finished and verified
🚫 blocked      - Cannot proceed
```

## Workflow

1. Read `WORKPADS.md`.
2. Load the active workpad's `tasks.md`, `knowledge.md`, and `references.md`.
3. Select a pending task based on dependencies, risk, and testability.
4. Mark it `🚧 in_progress`.
5. Do the work and record findings.
6. Mark it `✅ completed` only after acceptance criteria are met.
7. Add newly discovered tasks to `tasks.md`.
