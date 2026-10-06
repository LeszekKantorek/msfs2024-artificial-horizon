---
description: Repository instructions for implementation; read before working on a task.
status: active
---

## Implementation

- Read [README](../README.md) for runnable commands and user-facing limitations before implementation.
- Read [architecture](../docs/architecture.md) for target components and boundaries before implementation.
- Read the [telemetry contract](../docs/telemetry-contract.md) for telemetry or browser changes.
- Keep one Rust server and a lightweight HTML/CSS/JavaScript/SVG client.
- Read [iterative development](iterative-development.md) before choosing module boundaries or extending demo behavior.
- Keep reusable application logic in `src/lib.rs` and modules, with thin entry points under `src/bin/`.
- Use clap at the CLI boundary.
- Use `/health` for HTTP liveness, distinct from source validity and transport connectivity.
- Use Rust stable and target Windows x64 MSVC for the Rust library, CLI, server, and CI.
- Preserve read-only SSE; do not add browser simulator controls.
- Isolate Windows SimConnect dependencies.
- Keep demo/default builds independent of the simulator SDK.
- Never silently substitute demo data or valid-looking zero attitude after failure.
- Add focused tests for changed behavior.
- Report hardware checks honestly.
- Do not treat planned modules, commands, targets, or SDK compatibility as verified.
- Follow [CONTRIBUTING](../CONTRIBUTING.md) as the canonical source for workflow, document responsibilities, the definition of done, and required checks.
