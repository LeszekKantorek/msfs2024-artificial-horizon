---
description: Apply the package feature policy, shared Windows target, and mandatory local validation when changing builds or workflows.
status: active
---

* The package exposes no Cargo features; SimConnect is a Windows provider selected through runtime configuration.
* Dependency features remain valid and separate from the package feature policy.
* The library root rejects targets other than `x86_64-pc-windows-msvc`; do not add redundant platform gates to provider modules.
* Follow the shared target and Tests-before-Build-release sequence in [automated checks](../docs/testing/automated.md#automated-checks).
* The final build produces the release executable; debug tests and optimized release artifacts use separate profiles.
* Cargo explicitly names the executable `msfs2024-artificial-horizon`, with its entry point at `src/bin/main.rs`.
* Passing checks without a simulator does not establish SDK or simulator compatibility.
* Application compilation, Rust/JavaScript tests, and acceptance-validator tests run locally; local validation remains mandatory for the selected scope.
* Keep [PR acceptance](../.github/workflows/acceptance.yml) and its trusted-code boundary. Remove the `Build & Test` required status from the `main` ruleset while preserving `Acceptance criteria` and unrelated protections.
* Record actual validation evidence in the PR, a related issue, or a durable linked validation record. PR descriptions explain the change, rationale, and consequences without mandatory test-status sections, command lists, or testing checklists.
