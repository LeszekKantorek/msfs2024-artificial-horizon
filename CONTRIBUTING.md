# Contributing

Use English for code, comments, documentation, issues, commits, and pull requests.

## Issue-first workflow

1. Pick or create a GitHub issue with an outcome and acceptance criteria.
2. Check dependencies in [the roadmap](docs/roadmap.md).
3. Branch from current `main`: `feat/<issue>-<topic>`, `fix/<issue>-<topic>`, or
   `docs/<issue>-<topic>`. Repository preparation uses `chore/project-foundation`.
4. Implement one reviewable outcome and update affected documentation/contracts.
5. Open a pull request referencing the issue, describing changed behavior and
   validation. Use `Closes #N` only when all acceptance criteria are satisfied.

GitHub Issues are the source of truth for task status. The roadmap records order
and dependencies; avoid maintaining a second status checklist in repository files.
Unverified hardware acceptance stays open or becomes an explicit linked follow-up.

## Definition of done

- Acceptance criteria are met and relevant evidence is attached.
- Relevant automated checks pass; manual checks record environment and results.
- Errors, source loss, and reconnect behavior are considered for runtime changes.
- Changes to the telemetry contract include compatible fixtures and consumer updates.
- Documentation describes implemented behavior and clearly marks planned behavior.

Rust development and CI target Windows x64 MSVC only. Once Cargo is introduced,
the baseline commands are `cargo fmt --all -- --check`,
`cargo clippy --locked --all-targets -- -D warnings`, and `cargo test --locked`.
Run feature-specific checks as documented by the SimConnect integration; default
checks must not require the simulator. Add useful behavior tests, not assertions
that merely duplicate implementation details.

Keep Cargo.lock committed. Do not commit simulator SDK binaries, local SDK paths,
credentials, generated build output, or dependencies without distribution rights.
Preserve the existing MIT license. Record material architectural changes in docs/adr/.
