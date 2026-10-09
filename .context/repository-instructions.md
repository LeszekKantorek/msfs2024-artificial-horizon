---
description: Apply repository rules and acceptance criteria before implementation or validation.
status: active
---

## Steps

1. Read [README](../README.md), [architecture](../docs/architecture.md), and [CONTRIBUTING](../CONTRIBUTING.md) before implementation.
2. Select other sources for the affected scope: [contract](../docs/telemetry-contract.md), [tests](../docs/testing.md), or [iteration rules](iterative-development.md).
3. Distinguish target design and SDK assumptions from verified behavior.
4. Collect implementation findings in [#9](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/9), referencing the originating task.
5. Review those findings after that task closes before defining follow-up scope.

## Repository rules

* Use English for code, comments, documentation, knowledge entries, commit messages, and GitHub issues and PRs; follow the [writing style](writing-style.md) for project text, including GitHub comments.
* Commit `Cargo.lock`.
* Preserve the MIT license.
* Record material architecture changes in `docs/adr/`.
* Exclude SDK binaries, local SDK paths, credentials, generated output, and dependencies without distribution rights from commits.
* Target Windows x64 MSVC for Rust development and CI.
* Keep default checks independent of the simulator.
* Test behavior, not assertions that repeat implementation details.

## Definition of done

* [ ] Acceptance criteria have supporting evidence.
* [ ] Relevant [automated checks](../docs/testing.md#automated-checks) pass.
* [ ] Manual results identify the environment and observations separately from unavailable hardware checks.
* [ ] Runtime changes address errors, source loss, and reconnects.
* [ ] Telemetry changes include compatible fixtures and consumer updates.
* [ ] Documentation distinguishes implemented and planned behavior.

Sources: repository CONTRIBUTING rules, moved here by an accepted documentation decision on 2026-10-09, and the linked project documents.
