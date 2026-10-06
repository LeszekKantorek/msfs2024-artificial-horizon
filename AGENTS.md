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

## Project context

- Follow the document responsibilities in [CONTRIBUTING.md](CONTRIBUTING.md).
- Use [.context/index.md](.context/index.md) to find relevant project knowledge.
- Before choosing module boundaries or extending demo behavior, read
  [.context/iterative-development.md](.context/iterative-development.md).
- For other tasks, read only relevant `.context/` entries. Use `context-gather`
  for missing or conflicting knowledge and `context-apply` for known constraints.
- Check entry status and sources: follow `superseded_by` for superseded entries;
  do not apply deprecated entries. `active` describes current knowledge, not
  implementation progress or proof of a claim. Exclude `.context/sessions/`.
- Apply retrieved constraints to the actual change and its validation; reading
  or citing an entry alone is insufficient. Keep intended and observed behavior
  distinct, and surface unresolved conflicts rather than silently resolving them.
- Use `context-consolidate` to preserve new durable learning within the task's
  scope. Prefer updating its existing home or linking to a canonical source over
  duplicating documentation. Update `.context/index.md` when adding, renaming,
  or retiring entries. Do not require a memory update after every task.
- Use `context-review` when knowledge is conflicting, stale, duplicated, or hard
  to retrieve; do not require all four skills for every task.
- Keep progress, blockers, next steps, and test execution results in Issues/PRs
  or the task's own plan, not `.context/`. Do not store raw chats or secrets.
