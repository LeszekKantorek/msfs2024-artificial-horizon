# Task #91: DLL-backed Demo planning

This snapshot records the planning handoff. The amended Task #91 and its replacement Subtasks are the execution authority. Planning and review history below records the state when each document was written; historical publication-status statements are not current status. Proposed contracts and scenarios remain unverified until implementation provides evidence.

The original `.local/` package remains preserved. This branch copy is the active publication payload. Planning files may be committed on working branches but must be removed from the final tree before merging into `main`. The acceptance-rule change is prepared locally; tests and hosted deployment remain unverified.

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
| Keep scenario authoring and forward scenario-to-result verification; remove recorded-session conversion | Latest explicit response supersedes the earlier conversion requirement |
| Use only data available through the currently used SimConnect API | Explicit response; complete fictional Demo profile equivalence is replaced |
| Include API errors and malformed-packet injection | Explicit response |
| Use JSONL for Demo, including duration-based hold and ramp instructions | Explicit later clarification; replaces YAML serialization and enumerated snapshot content |
| DLL owns approximate timing | Explicit clarification; no hard real-time guarantee |
| Clock always advances; steps loop after the last step | Explicit correction; no clock reset, end waiting mode, or automatic lifecycle restoration |
| Demo DLL is a playback mechanism only | Pause and flight state follow scenario instructions; initial header state applies once |
| Scenarios may provide any subset of signals, events only, or no signal values | Explicit clarification; missing values do not fail registration and produce no invented observations |
| Unreceived instrument values default to null/unavailable, not zero | Explicit decision replacing the initial-zero preference; preserve existing SSE omission of absent optional fields |
| Adapt Demo to SimConnect behavior when they differ | Explicit latest clarification; preserve the common receiver and reproduce asynchronous arrival and notification |
| Send unchanged values according to the native request | Explicit clarification; flags 0 preserves equal samples, hold emits throughout its duration, and gap emits no samples for that signal |
| Simulate normal pause, active pause, active flight, and inactive flight | Explicit later addition |
| Provide several scenarios, including a narrated flight and separate potentially terminating errors | Explicit subsequent request |
| Keep a purpose-only scenario catalog without copying JSONL details | Explicit subsequent request |
| Validate the scenario after HTTP binds; fail setup before Open | Accepted later clarification; private setup may return E_FAIL and an actionable explanation |
| Accept the proposed basename/path restrictions, case insensitive | Explicit acceptance; DEMO.dll is valid |
| Verify pause delivery during implementation | Working assumption: messages continue and frozen values repeat; normal and active pause require separate native evidence |
| Stage rare fault coverage | Explicit clarification; preserve the inventory and identify implemented versus deferred cases |
| Consolidate screenshots and visual inspection into S8 | Explicit later clarification; retain applicable nonvisual tests in each Subtask, with no per-task appearance approval |
| Prepare branch files in `.plans/issue-91/` and the acceptance rule | Later explicit authorization supersedes the local-only location; no tests, builds, or application runs |

These decisions govern the amended execution handoff, subject to the explicit evidence gaps in the linked contracts.

## Read the proposal

| Document | Purpose |
| --- | --- |
| [Architecture](architecture.md) | Target boundaries, alternatives, consequences, and amendments |
| [Receive-path clarification](architecture.md#receive-path-clarification-after-the-astra-review) | Two-lane interaction, queue and buffer ownership, fidelity requirements, and superseded timing proposals |
| [Scenario and ABI contracts](contracts.md) | Proposed executable contract, JSONL records, timing, and error behavior |
| [Current evidence](evidence.md) | Revision, inspected sources, remote records, and evidence limits |
| [Delivery plan](plan.md) | Repository work first, then documentation and code outcomes |
| [Tasks, tests, and review](tasks-tests-review.md) | Per-task evidence and reviewer checks that connect each result to the prerequisites of the next stage |
| Assignment prompts: [implementation](prompts/implementation.md), [independent review](prompts/implementation-review.md), [Task acceptance](prompts/task-acceptance.md) | Copyable assignments with explicit validation and publication scope; each role checks its own outcome without repeating every earlier check |
| [Review](review.md) | Independent findings and added ideas distinguished from requested scope |
| [Astra review assessment](reviews/astra-01/assessment.md) | Current dispositions and links to the preserved independent review and frozen input |
| [Task amendment draft](task-91-amendment.md) | Local replacement direction and explicit changes to existing #123–#126 |
| [JSONL example](examples/hold-ramp-pause.jsonl) | Illustrative IAS holds/ramp, attitude, both pause modes, and flight activation |
| [Scenario catalog](demo-scenarios.md) | Main assumptions, test purpose, and expected completion for each scenario |
| [Existing error coverage](error-coverage.md) | Static native/host failure inventory and expected source/HTTP/application reactions |

Each file in `subtasks/` contains an issue body and a separate first-comment draft.
The comments carry draft execution context; the corresponding GitHub issue comments are the published handoff.
Seven Subtask bodies/comments are prepared. S6 now maps scenario inputs to output evidence; S7-converter is removed. Historical conversion drafts are retained only under sources/ and frozen reviews.
The example is a proposed scenario, not executable or parser-validated evidence. It is not the complete default scenario.
The additional files under `scenarios/` are concrete planning drafts and have not been executed or production-parser validated.

## Evidence limits

* Local HEAD and remote main were both `9356e47e481d422c3b771205cd7f3e82f07789b0` when inspected.
* The working tree was clean before these ignored local drafts.
* Earlier planning changed no tracked files. The later preparation changes acceptance tooling and its documentation, and adds this branch planning package. Application code, GitHub issues, and remote relationships remain unchanged.
* No tests, builds, application execution, conversion experiments, browser runs, or Markdown rendering were performed.
* Simulator and real-device acceptance remain separate from the proposed emulator checks.
