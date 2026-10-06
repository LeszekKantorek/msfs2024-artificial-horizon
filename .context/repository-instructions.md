---
description: Implementation and context-use rules for this repository; read before working on a task.
status: active
---

# Repository instructions

## Implementation

Read [README](../README.md) for runnable commands and user-facing limitations,
and [architecture](../docs/architecture.md) for target components and boundaries
before implementation. Read the [telemetry contract](../docs/telemetry-contract.md)
for telemetry or browser changes.

Apply the architectural constraints when changing code:

- Keep one Rust server and a lightweight HTML/CSS/JavaScript/SVG client.
- Keep reusable application logic in `src/lib.rs` and modules, with thin entry
  points under `src/bin/`. Use clap at the CLI boundary.
- Use `/health` for HTTP liveness, distinct from source validity and transport
  connectivity.
- Target Windows x64 MSVC for the Rust library, CLI, server, and CI.
- Preserve read-only SSE; do not add browser simulator controls.
- Isolate Windows SimConnect dependencies. Demo/default builds must not need
  the simulator SDK.
- Never silently substitute demo data or valid-looking zero attitude after failure.
- Add focused tests for changed behavior and report hardware checks honestly.
- Do not treat planned modules, commands, targets, or SDK compatibility as verified.

Follow [CONTRIBUTING](../CONTRIBUTING.md) for workflow, document responsibilities,
the definition of done, and required checks. It is the canonical source for those
rules.

## Project context

- Before choosing module boundaries or extending demo behavior, read
  [iterative development](iterative-development.md).
- Read only entries relevant to the task. Use `context-gather` for missing or
  conflicting knowledge and `context-apply` for known constraints.
- Check entry status and sources: follow `superseded_by` for superseded entries;
  do not apply deprecated entries. `active` describes current knowledge, not
  implementation progress or proof of a claim. Exclude `.context/sessions/`.
- Apply retrieved constraints to the actual change and its validation; reading
  or citing an entry alone is insufficient. Keep intended and observed behavior
  distinct, and surface unresolved conflicts rather than silently resolving them.
- Use `context-consolidate` to preserve new durable learning within the task's
  scope. Prefer updating its existing home or linking to a canonical source over
  duplicating documentation. Update [the index](index.md) when adding, renaming,
  or retiring entries. Do not require a memory update after every task.
- Use `context-review` when knowledge is conflicting, stale, duplicated, or hard
  to retrieve; do not require all four skills for every task.
- Keep progress, blockers, next steps, and test execution results in Issues/PRs
  or the task's own plan, not `.context/`. Do not store raw chats or secrets.
