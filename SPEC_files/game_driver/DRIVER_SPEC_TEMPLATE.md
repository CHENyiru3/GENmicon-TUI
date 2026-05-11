# <Driver ID> Driver Spec

Status: Draft
Owner: Maintainer
Last reviewed: 2026-05-10

## Purpose

Describe the reusable genre/runtime contract this driver provides.

## Source Anchors

Driver package:

- `examples/games/<game>/drivers/<driver-id>/<version>/driver.toml`

Runtime code:

- `crates/game/src/driver.rs`
- `crates/game/src/script.rs`
- `crates/game/src/agents.rs`

Affected games:

- `SPEC_files/games/<game>.md`

## Maintainer Prompt

```markdown
Spec: SPEC_files/game_driver/drivers/<driver-id>.md
Goal:
Games affected:
Current behavior:
Desired behavior:
Driver manifest changes:
Script function changes:
Agent role/template changes:
Acceptance criteria:
Validation I expect:
```

## Driver Contract

- Driver ID:
- Current version:
- Runtime:
- Default topology:
- Entry skill:
- Declared functions:
- Default roles:
- Maximum active roles:

## Compatibility

- Which game versions depend on this driver.
- Whether save reload requires the exact driver version.
- Whether a driver version bump is required.

## Acceptance Criteria Checklist

- [ ] `driver.toml` matches this spec.
- [ ] Scripts and declared functions match this spec.
- [ ] Agent templates and role bounds match this spec.
- [ ] Affected game specs are updated.
- [ ] Runtime tests cover the changed behavior.
