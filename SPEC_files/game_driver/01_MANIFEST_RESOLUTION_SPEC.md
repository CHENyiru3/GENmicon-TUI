# Driver Manifest And Resolution Spec

Status: Active
Owner: Maintainer
Last reviewed: 2026-05-10

## Purpose

This spec owns `driver.toml`, installed driver layout, semver resolution,
manifest validation, warnings, and save driver locks.

## Source Anchors

Primary code:

- `crates/game/src/driver.rs`
- `crates/game/src/manifest.rs`
- `crates/game/src/paths.rs`
- `crates/game/src/save.rs`

Example manifests:

- `examples/games/reconciliation-demo/drivers/galgame/0.1.0/driver.toml`
- `examples/games/thirteen-angry-man/drivers/deliberation-drama/0.1.0/driver.toml`
- `examples/games/reconciliation-demo/game.toml`
- `examples/games/thirteen-angry-man/game.toml`

## Current Behavior

- Driver packages live under `<root>/<driver-id>/<version>/driver.toml`.
- Game manifests specify `[driver].id` and `[driver].version`.
- Version requirements such as `^0.1` resolve to the highest matching installed
  driver version.
- Reload can resolve exact driver versions recorded in saves.
- Missing optional files can produce warnings; invalid manifests fail.

## Design Principles

- A game can depend on a semver range, but a save should record a concrete
  version.
- Driver paths must stay under the driver root.
- Manifest validation should fail early with clear errors.
- Warnings are acceptable for optional files; unsafe paths are not.

## Acceptance Criteria Checklist

- [ ] `driver.id` is validated with the same filesystem-safe ID rules as game
      IDs.
- [ ] `driver.version` parses as a concrete semver version.
- [ ] Game driver requirements parse and resolve predictably.
- [ ] Save driver locks can be checked without modifying the save.
- [ ] Manifest warnings identify missing optional files clearly.

## Validation Gates

- `driver_resolver_selects_highest_matching_version_and_exact_reload`
- `driver_resolver_rejects_manifest_version_mismatch`
- Related `crates/game` manifest/save tests.
