# Task #91: DLL-backed Demo planning

Status: Local discussion draft. No implementation or GitHub publication is authorized.

## Recommended result

Select `--source demo.dll` or `--source SimConnect.dll`.
Both selections use the existing SimConnect receiver, decoder, lifecycle rules, normalization, Publisher, HTTP/SSE, and EFI.
The project-owned Demo DLL supplies the currently consumed SimConnect API.

Demo scenarios use JSONL instructions such as a five-second hold followed by a five-second ramp.
The DLL owns the monotonic clock and generates requested observations during those phases.
This change exercises native registration names, units, indexes, request routing, message decoding, and source lifecycle.

## Discussion decisions

| Decision | Authority |
| --- | --- |
| Keep scenario authoring and recorded-session conversion | Explicit response in this conversation on 2026-11-07 |
| Use only data available through the currently used SimConnect API | Explicit response; complete fictional Demo profile equivalence is replaced |
| Include API errors and malformed-packet injection | Explicit response |
| Use JSONL for Demo, including duration-based hold and ramp instructions | Explicit later clarification; replaces YAML serialization and enumerated snapshot content |
| DLL owns approximate timing | Explicit clarification; no hard real-time guarantee |
| Simulate normal pause, active pause, active flight, and inactive flight | Explicit later addition |
| Provide several scenarios, including a narrated flight and separate potentially terminating errors | Explicit subsequent request |
| Keep a purpose-only scenario catalog without copying JSONL details | Explicit subsequent request |
| Work only in `.local/`; static analysis only | Explicit restriction; no tests, builds, or application runs |

These decisions authorize planning. They do not publish amendments to #91 or change existing Subtasks.

## Read the proposal

| Document | Purpose |
| --- | --- |
| [Architecture](architecture.md) | Target boundaries, alternatives, consequences, and amendments |
| [Scenario and ABI contracts](contracts.md) | Proposed executable contract, JSONL records, timing, and error behavior |
| [Current evidence](evidence.md) | Revision, inspected sources, remote records, and evidence limits |
| [Delivery plan](plan.md) | Repository work first, then documentation and code outcomes |
| [Review](review.md) | Independent findings and added ideas distinguished from requested scope |
| [Task amendment draft](task-91-amendment.md) | Local replacement direction and explicit changes to existing #123–#126 |
| [JSONL example](examples/hold-ramp-pause.jsonl) | Illustrative IAS holds/ramp, attitude, both pause modes, and flight activation |
| [Scenario catalog](demo-scenarios.md) | Main assumptions, test purpose, and expected completion for each scenario |
| [Existing error coverage](error-coverage.md) | Static native/host failure inventory and expected source/HTTP/application reactions |

Each file in `subtasks/` contains an issue body and a separate first-comment draft.
The comments carry execution context. No comments have been sent to GitHub.
Seven Subtask bodies/comments are prepared, including S8 for the requested scenario suite. Converter implementation S7 remains deferred until S6 establishes its revised contract.
The example is a proposed scenario, not executable or parser-validated evidence. It is not the complete default scenario.
The additional files under `scenarios/` are concrete planning drafts and have not been executed or production-parser validated.

## Evidence limits

* Local HEAD and remote main were both `9356e47e481d422c3b771205cd7f3e82f07789b0` when inspected.
* The working tree was clean before these ignored local drafts.
* No application code, tracked documentation, repository configuration, issues, or relationships were changed.
* No tests, builds, application execution, conversion experiments, browser runs, or Markdown rendering were performed.
* Simulator and real-device acceptance remain separate from the proposed emulator checks.
