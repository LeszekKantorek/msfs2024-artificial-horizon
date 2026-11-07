# S1: Prepare the workspace for coordinated EXE and Demo DLL builds

## Issue body

Parent: Task #91 under Epic #9. Prerequisites: no earlier Subtask; accept the revised architecture direction first.

Execution: one GPT-6.1 Terra developer at medium. Use Sol Medium consultation for consequential gaps.
Read Task #91, Epic #9, this body, its first context comment, and prerequisite results.
Refresh issue artifacts and repository state before execution.
Follow the repository execution/reporting procedure; report results and gaps in the main conversation.
This is a local draft. It grants no implementation or publication authorization.

### Outcome

One SDK-free Windows/MSVC workspace builds the existing application and a project-owned Demo DLL artifact.
Application behavior remains unchanged during this repository preparation.

### Scope

* Add one workspace member at `crates/demo/`; keep the root application package and its binary name.
* Configure the member to produce `demo.dll`; preserve MIT licensing, committed lockfile, and no package feature switches.
* Update build instructions and artifact expectations.
* Exclude SimConnect emulation, scenario implementation, CLI changes, release installers, and GitHub workflow/ruleset changes.

### Context

* Current root `Cargo.toml`, `Cargo.lock`, `.gitignore`, and `src/lib.rs`.
* `docs/architecture.md#library-and-platform-constraints`.
* `docs/testing/automated.md#automated-checks` and `CONTRIBUTING.md#environment-setup`.
* ADR 0006 and the accepted architecture amendment: one additional package is justified by the actual DLL artifact.

### Implementation guidance

1. Retain the root package and add workspace membership/default membership deliberately.
2. Create the smallest Demo library target with no pretend SimConnect API stubs.
3. Confirm artifact naming and output locations for debug, release, and test integration.
4. Document workspace-wide checks and the preliminary debug DLL build needed by later loader tests.
5. Keep official SDK binaries external and generated output ignored.

### Acceptance criteria

* [ ] Both packages resolve on Windows x64 MSVC without a simulator, SDK, import library, or new external service.
* [ ] A complete release build emits the existing named EXE and `demo.dll` in the same target/profile directory.
* [ ] The existing application retains its current startup behavior; the new DLL is not presented as a working source yet.
* [ ] Build/test documentation distinguishes the application EXE directory from hashed test-executable directories.
* [ ] No generated DLL, SDK file, local path, credential, or local plan enters Git.
* [ ] Existing PR acceptance workflow behavior and unrelated repository protections remain unchanged.

### Validation

Later execution: workspace metadata/build inspection, applicable complete Windows checks, and independent review of build/configuration changes.
Record both artifact paths and identities.
Do not add a unit test that merely repeats an empty manifest.
No tests or builds were performed while preparing this draft.

## First comment: execution context

The original #91 plan assumed one package and a direct Demo provider.
The revised design needs one additional native artifact; it does not need a plugin framework.

Use `[lib]` naming that produces `demo.dll` on the supported target.
Cargo normally places a workspace member's DLL in the common target/profile output; verify that actual result.
Do not promise `cargo run` builds an unrelated workspace cdylib automatically.

Later loader integration tests must receive an explicit absolute DLL path.
A hashed test binary under `debug/deps/` is not the application EXE and must not drive deployment discovery.

Follow the current contribution worktree procedure for later authorized execution.
Planning remains in the main checkout's ignored `.local/` only.
