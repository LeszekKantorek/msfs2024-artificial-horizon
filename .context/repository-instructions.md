---
description: Implementation rules for this repository; read before working on a task.
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
- Use Rust stable and target Windows x64 MSVC for the Rust library, CLI, server, and CI.
- Preserve read-only SSE; do not add browser simulator controls.
- Isolate Windows SimConnect dependencies. Demo/default builds must not need
  the simulator SDK.
- Never silently substitute demo data or valid-looking zero attitude after failure.
- Add focused tests for changed behavior and report hardware checks honestly.
- Do not treat planned modules, commands, targets, or SDK compatibility as verified.

Follow [CONTRIBUTING](../CONTRIBUTING.md) for workflow, document responsibilities,
the definition of done, and required checks. It is the canonical source for those
rules.
