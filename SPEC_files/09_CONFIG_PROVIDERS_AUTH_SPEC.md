# Config, Providers, And Auth Spec

Status: Active
Owner: Maintainer
Last reviewed: 2026-05-10

## Purpose

This spec owns configuration loading, profiles, provider selection, model
selection, credentials, auth commands, config UI, and precedence between config
files, keyring, environment variables, and CLI overrides.

## Source Anchors

Primary code:

- `crates/tui/src/config.rs`
- `crates/tui/src/settings.rs`
- `crates/tui/src/config_ui.rs`
- `crates/tui/src/commands/config.rs`
- `crates/tui/src/commands/provider.rs`
- `crates/tui/src/models.rs`

Related code:

- `crates/config/src/lib.rs`
- `crates/secrets/src/lib.rs`
- `crates/tui/src/tui/model_picker.rs`
- `crates/tui/src/tui/provider_picker.rs`
- `crates/tui/src/auto_reasoning.rs`
- `crates/tui/src/pricing.rs`
- `config.example.toml`

Canonical docs:

- `docs/CONFIGURATION.md`
- `README.md`
- `README.zh-CN.md`

Tests and fixtures:

- Config/provider unit tests where present
- TUI picker tests where present

## Maintainer Prompt

```markdown
Spec: SPEC_files/09_CONFIG_PROVIDERS_AUTH_SPEC.md
Goal:
Config/provider/auth surface affected:
Current behavior:
Desired behavior:
Precedence requirements:
Security or secret-handling requirements:
Acceptance criteria:
Validation I expect:
```

## Current Behavior

- API keys can be saved to config/keyring or supplied by environment variables.
- Saved config keys take precedence over keyring and environment where
  documented.
- Provider presets include DeepSeek and other OpenAI-compatible options.
- Model and provider pickers expose runtime selection in the TUI.
- `[game]` config remains reserved/planned until loader and `/config` UI support
  it.

## Design Principles

- Secret values must not be printed in logs, errors, or docs.
- Config precedence must be predictable and documented.
- Provider changes need matching model, pricing, auth, and UI behavior.
- Reserved config surfaces should not be documented as shipped.

## Change Workflow

- Map the change to config parse, config persistence, auth command, picker UI,
  provider request construction, or docs.
- Update `config.example.toml` when adding a shipped config key.
- Update docs and localization/help when user-facing command text changes.
- Add tests for precedence and backward compatibility.

## Acceptance Criteria Checklist

- [ ] Config key or provider behavior is parsed and applied correctly.
- [ ] Precedence is tested and documented.
- [ ] Secrets are redacted from display and logs.
- [ ] UI pickers and command help match behavior.
- [ ] Reserved/planned config is not presented as shipped.

## Validation Gates

- Targeted config/provider tests.
- `cargo test -p deepseek-tui --all-features`.
- Full workspace tests for extracted config or secrets crate changes.

## Risks

- Auth precedence mistakes can make users think a revoked key is still active.
- Provider additions can require pricing, model picker, docs, and retry updates.
- Config docs can accidentally advertise unimplemented behavior.
