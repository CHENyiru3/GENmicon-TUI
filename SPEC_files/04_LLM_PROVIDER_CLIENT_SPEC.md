# LLM Provider Client Spec

Status: Active
Owner: Maintainer
Last reviewed: 2026-05-10

## Purpose

The LLM provider client owns model/provider resolution, OpenAI-compatible Chat
Completions requests, DeepSeek streaming behavior, reasoning-content handling,
usage parsing, retry/status reporting, pricing, and auto-routing.

## Source Anchors

Primary code:

- `crates/tui/src/client.rs`
- `crates/tui/src/client/chat.rs`
- `crates/tui/src/llm_client/mod.rs`
- `crates/tui/src/models.rs`
- `crates/tui/src/auto_reasoning.rs`

Related code:

- `crates/tui/src/pricing.rs`
- `crates/tui/src/retry_status.rs`
- `crates/tui/src/cost_status.rs`
- `crates/agent/src/lib.rs`
- `crates/config/src/lib.rs`

Canonical docs:

- `README.md`
- `docs/CONFIGURATION.md`
- `docs/ARCHITECTURE.md`

Tests and fixtures:

- `crates/tui/src/llm_client/mock.rs`
- Unit tests beside provider/model modules

## Maintainer Prompt

```markdown
Spec: SPEC_files/04_LLM_PROVIDER_CLIENT_SPEC.md
Goal:
Provider/model affected:
Current behavior:
Desired behavior:
API compatibility requirements:
Cost or usage display expectations:
Acceptance criteria:
Validation I expect:
```

## Current Behavior

- DeepSeek V4 model IDs are `deepseek-v4-pro` and `deepseek-v4-flash`.
- Legacy `deepseek-chat` and `deepseek-reasoner` remain compatibility aliases
  for `deepseek-v4-flash`.
- The documented API path is OpenAI-compatible Chat Completions.
- `auto` chooses concrete model and thinking settings locally before the real
  turn.
- DeepSeek thinking blocks stream before final answer content and must be
  replayed correctly when tool calls occur.

## Design Principles

- Keep provider behavior explicit and observable.
- Do not send local-only pseudo-models such as `auto` to upstream APIs.
- Preserve compatibility aliases unless removal is explicitly approved.
- Treat pricing and token usage as approximate unless verified by provider
  response semantics.

## Change Workflow

- Check config/provider surfaces before changing request construction.
- Update model picker, provider picker, config docs, and README when model
  behavior changes.
- Use mock LLM tests for retry, streaming, usage parsing, and reasoning replay.
- Verify external API claims against official provider docs when adding or
  changing provider-specific behavior.

## Acceptance Criteria Checklist

- [ ] Request URL, model ID, thinking setting, and headers match the selected
      provider.
- [ ] Streaming parser handles thinking, content, tool calls, errors, and usage.
- [ ] Retry/status messages are clear and do not leak secrets.
- [ ] Cost and token display are updated when usage semantics change.
- [ ] Docs and config examples match actual behavior.

## Validation Gates

- Targeted client/model tests.
- `cargo test -p deepseek-tui --all-features`.
- Manual smoke test with mocked or real credentials only when appropriate.

## Risks

- Reasoning-content replay mistakes can produce HTTP 400 responses on DeepSeek
  thinking models.
- Provider-specific compatibility paths can drift from docs and config UI.
- Token accounting bugs can inflate session cost estimates.
