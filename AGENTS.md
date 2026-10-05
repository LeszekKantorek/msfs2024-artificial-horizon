# Repository instructions

- Write code, comments, documentation, issues, commits, and PRs in English.
- Read README.md and docs/architecture.md before implementation. Read
  docs/telemetry-contract.md for telemetry or browser changes.
- Track work in GitHub Issues. Work on a topic branch, not directly on main.
- Keep one Rust server and a lightweight HTML/CSS/JavaScript/SVG client.
- Initialize Rust as a library package; keep reusable logic in `src/lib.rs` and
  modules, with thin entry points under `src/bin/`. Use clap at the CLI boundary.
- Use `/health` for HTTP liveness.
- Target Windows x64 MSVC only for the Rust library, CLI, server, and CI.
- Preserve the read-only SSE architecture; do not add browser simulator controls.
- Isolate Windows SimConnect dependencies. Demo/default builds must not need the SDK.
- Never silently substitute demo data or valid-looking zero attitude after failure.
- Keep source validity distinct from HTTP health and transport connectivity.
- Add focused tests for changed behavior and report hardware checks honestly.
- Use CONTRIBUTING.md for the definition of done and baseline checks once Cargo exists.
- Do not treat planned modules, commands, targets, or SDK compatibility as verified.
