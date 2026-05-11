# Runtime API, Tasks, And Automation Spec

Status: Active
Owner: Maintainer
Last reviewed: 2026-05-10

## Purpose

This spec owns headless and background operation: HTTP/SSE runtime API,
durable thread events, background task queues, gates, artifacts, PR attempts,
and scheduled automation.

## Source Anchors

Primary code:

- `crates/tui/src/runtime_api.rs`
- `crates/tui/src/runtime_threads.rs`
- `crates/tui/src/task_manager.rs`
- `crates/tui/src/automation_manager.rs`

Related code:

- `crates/app-server/src/lib.rs`
- `crates/app-server/src/main.rs`
- `crates/tui/src/tools/tasks.rs`
- `crates/tui/src/tools/automation.rs`
- `crates/tui/src/commands/task.rs`
- `crates/tui/src/commands/queue.rs`
- `crates/tui/src/commands/jobs.rs`

Canonical docs:

- `docs/RUNTIME_API.md`
- `docs/OPERATIONS_RUNBOOK.md`
- `docs/ARCHITECTURE.md`

Tests and fixtures:

- Runtime API and task manager unit/integration tests where present
- `crates/tui/tests/`

## Maintainer Prompt

```markdown
Spec: SPEC_files/08_RUNTIME_API_TASKS_AUTOMATION_SPEC.md
Goal:
API/task/automation surface affected:
Current behavior:
Desired behavior:
Persistence and recovery expectations:
External visibility:
Acceptance criteria:
Validation I expect:
```

## Current Behavior

- `deepseek serve --http` exposes local HTTP/SSE runtime behavior.
- Runtime threads store durable thread, turn, item, and event records.
- The task manager owns background queues, gates, artifacts, and PR attempts.
- Automation manager schedules recurring runs.

## Design Principles

- Runtime API behavior must be stable, documented, and version-aware.
- Background work should be observable and recoverable.
- Externally visible actions such as PR comments or closures require explicit
  policy review and maintainer intent.
- Durable state changes need migration and recovery thinking.

## Change Workflow

- Identify whether the change affects API contract, event schema, task state,
  scheduler behavior, or TUI presentation.
- Update `docs/RUNTIME_API.md` for endpoint, payload, or SSE changes.
- Update operations docs for behavior that affects deployment or recovery.
- Add tests for event persistence and task lifecycle transitions.

## Acceptance Criteria Checklist

- [ ] API request/response and SSE event contracts are documented.
- [ ] Task state transitions survive restart when required.
- [ ] Gates and artifacts are visible to users or API clients.
- [ ] Automation schedules do not run unexpectedly or duplicate work.
- [ ] External side effects are approval-gated or explicitly requested.

## Validation Gates

- Targeted runtime API tests.
- Targeted task/automation tests.
- `cargo test -p deepseek-tui --all-features`.
- Full workspace tests for `crates/app-server` protocol changes.

## Risks

- Schema drift can break clients even when the TUI still works.
- Background task bugs can continue after the visible turn ends.
- Automation changes can create repeated external side effects.
