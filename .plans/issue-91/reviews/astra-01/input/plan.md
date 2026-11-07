# Deliver the revised Task #91

Status: Local Subtask drafts. Publication, execution, and existing-issue changes remain unauthorized.

## Assignments and dependencies

Planning and consultation use GPT-6.1 Sol at medium.
Implementation defaults to one GPT-6.1 Terra developer at medium.
Independent review remains separate from authorship.
This session used Sol subagents for analysis; it did not execute a Terra assignment.

| ID | Draft outcome | Kind | Prerequisites |
| --- | --- | --- | --- |
| S1 | [Prepare the workspace and coordinated artifact build](subtasks/S1-repository.md) | Repository first | Architecture direction accepted; no prerequisite Subtask |
| S2 | [Record the revised contracts and decision](subtasks/S2-documentation.md) | Documentation | S1; discussion amendments accepted |
| S3 | [Build the healthy JSONL SimConnect Demo DLL](subtasks/S3-demo-dll.md) | Code | S2 |
| S4 | [Select either DLL through the common application receiver](subtasks/S4-application.md) | Code and current usage | S3 |
| S5 | [Add API and packet fault scenarios](subtasks/S5-faults.md) | Code | S4 |
| S6 | [Establish the raw-recording conversion contract](subtasks/S6-conversion-evidence.md) | Bounded investigation | S2 |
| S7 | Implement session conversion and converted-file replay | Deferred code | S4 and accepted S6 result |
| S8 | [Deliver narrated flight and focused scenario suite](subtasks/S8-scenario-suite.md) | Scenario data, integration tests, and catalog | S4 and S5 |

S1 changes repository structure before functional source changes.
S2 promotes documentation before dependent implementation.
S3–S5 deliver the common native path and requested lifecycle/fault behavior.
S6 may establish its result without code, a PR, or project execution.
S8 adds the explicitly requested multi-scenario outcome; it does not replace routine unit/boundary testing with one long narrative.
No parallel developers are proposed.

```mermaid
flowchart LR
    S1["S1 repository"] --> S2["S2 contracts"]
    S2 --> S3["S3 DLL"]
    S3 --> S4["S4 application"]
    S4 --> S5["S5 faults"]
    S2 --> S6["S6 conversion evidence"]
    S6 --> S7["S7 converter: deferred"]
    S4 --> S7
    S5 --> S8["S8 scenario suite"]
```

## Existing Subtasks

Do not execute #123–#126 unchanged against this proposal.
Their current YAML/normalized-publication instructions conflict with the revised boundary.

| Existing issue | Useful retained intent | Required future treatment |
| --- | --- | --- |
| #123 | External scenario data; default scenario migration | Replace YAML/full snapshot ownership with the selected JSONL/native-instruction contract after publication is authorized |
| #124 | Selection, timing, application integration, shared clients | Replan around DLL selection and the common receiver |
| #125 | Conversion evidence and input interpretation | Preserve its established old mapping; reopen only the new raw-boundary question in S6 |
| #126 | Explicit session conversion and integrated playback | Defer revised implementation until S6 establishes support and fidelity |

These are proposed dispositions, not instructions to edit historical records automatically.
Before publication, refresh complete bodies, comments, artifacts, relationships, and any execution results.
Choose whether to amend open children or create replacements only under explicit authorization.

## Repository changes

Recommended target layout:

```text
Cargo.toml                       existing application package plus workspace
crates/demo/Cargo.toml            focused project-owned cdylib package
crates/demo/src/                  JSONL reader, timeline, ABI subset, native encoding
demo_scenario.jsonl               compact default instructions, embedded into demo.dll
demo-scenarios/                   narrated flight, lifecycle, availability, and isolated faults
docs/sources/demo-scenarios.md    purpose-only catalog; JSONL owns the scenario details
tests/fixtures/demo/              synthetic scenarios and independent expected observations
src/providers/simconnect/         current shared receiver
src/application.rs               typed DLL identity and configuration
src/bin/main.rs                   DLL-path source selection
src/bin/demo-convert.rs           only after S6; thin converter command
```

Use existing focused source files where possible.
Do not add a shared SDK crate, provider registry, installer, generic release pipeline, or new GitHub build workflow.

## Acceptance coverage

| Requirement | Owning outcome and later evidence |
| --- | --- |
| EXE and Demo DLL built without SDK | S1: Windows workspace builds and artifact identities |
| Compact JSONL and 5-second hold/ramp | S3: controlled clock checkpoints plus actual DLL loading |
| All used signals, units, indexes, IDs, diagnostic shape | S2/S3: independent catalog and registration tests |
| Normal/active pause and active/inactive flight | S3/S4: native events, query state, stale/waiting/paused/live recovery |
| Same native acquisition path | S4: default/explicit selection through real loader and SSE |
| API failures and malformed packets | S5: deterministic scenarios, memory-safe test buffers, source-failure isolation |
| Additional existing native errors | S5: explicit source/HTTP/application outcomes; host failures use harness conditions |
| Flight tour and multiple focused scenarios | S8: coherent progression, both pauses, simulator availability, and isolated terminal failures |
| Scenario documentation without duplication | S2 establishes ownership; S8 verifies catalog links/purposes against committed files |
| Recorded-session conversion retained | S6 establishes mapping; S7 later implements it |
| EFI presentation capability preserved | S4: retained independent fixtures; narrower source output explicitly distinguished |
| Complete application delivery | Each code/build outcome: coordinated checks/build; final integrated EXE plus demo.dll |

## Validation instructions for later implementation

The current static-only restriction applies to this planning session.
The commands below are proposed later checks, not reported results or authorization to run them now.

Use Windows x64 MSVC and the current validation matrix.
After S1, use workspace-wide variants where appropriate:

```powershell
$env:RUSTUP_TOOLCHAIN = 'stable'
$env:CARGO_BUILD_TARGET = 'x86_64-pc-windows-msvc'
cargo fmt --all -- --check
cargo clippy --workspace --locked --all-targets -- -D warnings
cargo build --workspace --locked
cargo test --workspace --locked --all-targets
node --test tests/*.test.mjs
cargo build --workspace --locked --release
```

The preliminary debug build supplies the DLL for loader integration tests.
It does not replace the final complete release build.
Ensure tests select its explicit absolute path rather than assuming the DLL sits beside a hashed test executable.

For affected display/runtime integration, run both desktop scripts against the complete release delivery.
Update their old CLI assumptions first and inspect the required matched captures.
Use controlled scheduler tests for precise timing; real thread scheduling supports bounded timing evidence rather than exact timestamps.

Record the revision, local changes, commands, environment, EXE and DLL hashes, scenario identity, observed outputs, review, and missing checks.
Do not add builds/tests to GitHub Actions.
No current acceptance checkbox is marked complete by this plan.

## Deferred converter assignment

S7 intentionally has no implementation-ready body yet.
The earlier normalized YAML mapping cannot be reused unchanged for raw DLL inputs.

S6 must establish supported schemas, recoverable fields, grouping, timing tolerances, lifecycle mapping, loss reporting, and rejection behavior.
Then Sol prepares S7's complete six-field body and context comment from that accepted result.
Do not silently drop conversion from Task completion while this outcome is deferred.
