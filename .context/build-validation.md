---
description: Apply the package feature policy and shared Windows target when changing builds or CI.
status: active
---

* The package exposes no Cargo features; SimConnect is a Windows provider selected through runtime configuration.
* Dependency features remain valid and separate from the package feature policy.
* The library root rejects targets other than `x86_64-pc-windows-msvc`; do not add redundant platform gates to provider modules.
* Follow the shared target and Tests-before-Build-release sequence in [automated checks](../docs/testing/automated.md#automated-checks).
* The final build produces the release executable; debug tests and optimized release artifacts use separate profiles.
* Cargo explicitly names the executable `msfs2024-artificial-horizon`, with its entry point at `src/bin/main.rs`.
* The current SimConnect module is a placeholder and does not establish SDK or simulator compatibility.
* Measure local command time separately from GitHub runner setup.
* CI uses no cross-run Rust cache action, as requested on 2026-10-10.
* When publishing the renamed `Build & Test` job, check whether repository rules still require the old `check` status.

Source: accepted build optimization scope, 2026-10-10; implementation in [CI](../.github/workflows/ci.yml) and [architecture](../docs/architecture.md#boundaries).
