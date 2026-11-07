# Static evidence for the revised Task #91

## Revision and working state

| Source | Observed state |
| --- | --- |
| Local HEAD | `9356e47e481d422c3b771205cd7f3e82f07789b0` |
| Remote main | Same SHA through the GitHub branch reader |
| Tracked working tree | Clean before local drafting |
| Planning date | 2026-11-07, Europe/Warsaw |
| Local output | Git-ignored `.local/issue-91-dll/` |
| Method | Static source/document inspection and read-only remote retrieval |

## GitHub context

| Record | Retrieved context |
| --- | --- |
| [Task #91](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/91) | Open; updated 2026-11-06T22:10:20Z; body, metadata, comments, parent, child listing, and separate artifact listing read |
| [Accepted comment](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/91#issuecomment-6696677478) | Complete content read; updated 2026-11-06T22:10:20Z |
| Task artifacts | Successful page 1 with zero items; no artifact URLs or update timestamps to cite |
| [Epic #9](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/9) | Confirmed actual parent; updated 2026-11-06T21:07:05Z; body, comments, and separate artifact listing read |
| Epic comments/artifacts | Successful empty lists |
| Existing children | #123, #124, #125, #126; all open in the child listing |
| Child results | No implementation-completion evidence established in this analysis; refresh complete child context before any authorized reorganization or execution |

Original Task and accepted-comment bodies are preserved verbatim under `sources/`.
The metadata snapshot preserves the retrieved child descriptions.
No original plan was silently replaced.
No artifacts were executed.

## Implementation findings

Line references identify the inspected revision. Re-locate the named symbols if they move.

| Finding | Checked source |
| --- | --- |
| CLI selects enum Demo/Simconnect and separates --dll | `src/bin/main.rs:12–63` |
| Composition fixes source, binds HTTP, then starts selected producer | `src/application.rs:17–29, 61–83, 118–154` |
| Direct Demo constructs normalized values and publishes without native calls | `src/providers/demo.rs:26–107, 180–198` |
| Current Demo includes fictional full profile and direct acceleration | `src/providers/demo.rs:95–107, 154–164` |
| Native consumer has seventeen independent optional field pairs | `src/providers/simconnect/native.rs:30–48` |
| Eight resolved exports already support an explicitly selected DLL | `src/providers/simconnect/native.rs:125–146, 267–274` |
| Logging changes attitude definition from two fields to six probes | `src/providers/simconnect/native.rs:345–387` |
| Native receiver rejects a Demo publisher today | `src/providers/simconnect/runtime.rs:102–106` |
| Native worker configures real lifecycle/data requests | `src/providers/simconnect/runtime.rs:517–597, 615–651` |
| Receipt time is assigned after dispatch | `src/providers/simconnect/runtime.rs:686` |
| Decoder failures are fatal; native dispatch HRESULT errors reconnect | `src/providers/simconnect/runtime.rs:675–683` |
| Lifecycle separates connection, Sim, pause, loaded flight, and fresh sample | `src/providers/simconnect/lifecycle.rs:27–32, 49–120` |
| Optional data builds only a partial profile and derives acceleration | `src/providers/simconnect/instruments.rs:9–65, 101–107` |
| Current journal emits schema 4 | `src/providers/simconnect/journal.rs:110–111` |
| Raw attitude and instrument observations are journaled | `src/providers/simconnect/runtime.rs:763–815` |
| No workspace or DLL target exists | Root `Cargo.toml` |
| Browser runtime runner assumes old CLI | `tests/browser/pfd-runtime-browser.mjs:22, 265` |
| Several integration tests depend on direct Demo helpers | `tests/telemetry.rs`, `tests/airspeed_ranges.rs`, `tests/altitude.rs`, `tests/heading.rs`, and `tests/http_sse.rs` |

## Official API references

These documents support contract interpretation, not binary conformance:

* [AddToDataDefinition](https://docs.flightsimulator.com/msfs2024/retail/programming-apis/simconnect/api-reference/events-and-data/simconnect_addtodatadefinition/) specifies client definitions, indexed names, units, and datatypes. SimVar names are case insensitive.
* [GetNextDispatch](https://docs.flightsimulator.com/msfs2024/retail/programming-apis/simconnect/api-reference/general/simconnect_getnextdispatch/) describes returned native buffers and lengths.
* [Open](https://docs.flightsimulator.com/msfs2024/retail/programming-apis/simconnect/api-reference/general/simconnect_open/) describes the notification handle.
* [System-event subscriptions](https://docs.flightsimulator.com/html/Programming_Tools/SimConnect/API_Reference/Events_And_Data/SimConnect_SubscribeToSystemEvent.htm) describe Pause_EX1 modes. Normal pause uses flag 1; active pause uses flag 4.
* [System-state requests](https://docs.flightsimulator.com/html/Programming_Tools/SimConnect/API_Reference/General/SimConnect_RequestSystemState.htm) describe Sim and FlightLoaded queries.
* [Native units](https://docs.flightsimulator.com/msfs2024/retail/programming-apis/simvars/simulation-variable-units/) describe nonzero Bool true, including -1.
* [Autopilot variables](https://docs.flightsimulator.com/msfs2024/retail/programming-apis/simvars/aircraft-simvars/aircraft-autopilot-and-assistant-variables/) and [aircraft system variables](https://docs.flightsimulator.com/msfs2024/retail/programming-apis/simvars/aircraft-simvars/aircraft-system-variables/) were inspected by the native-analysis subagent. Exact indexed-read ambiguities remain explicit.

The attempted older MSFS2024 HTML paths for three API pages returned 404.
The current retail API pages were then retrieved successfully.
No external SDK header or binary was obtained or executed.

## Analysis consultations

Two GPT-6.1 Sol subagents at medium performed static consultations:

* Native boundary analysis established that a drop-in DLL can reuse the receiver, but source identity, package policy, and full Demo equivalence need explicit changes.
* Clarification and proportionality analysis identified the YAML/replay scope conflict, field-subset difference, fault scope, and timing-model alternatives.
* Later consultation distinguished authored holds, gaps, and recorded observations. It recommended independent tracks for existing simultaneous cycles.
* These are analysis results. They do not establish that a proposed engine, DLL, or converter works.

## Missing evidence

* No Rust or JavaScript tests were run.
* No application or DLL was built or loaded.
* No scenario or converter prototype was executed.
* No browser, screenshot, simulator, or phone checks were performed.
* No Markdown or Mermaid rendering was performed.
* Actual schema-5 records and revised raw-conversion behavior remain unverified.
* Exact ABI layouts and every per-variable index/unit need independent confirmation during implementation preparation.

## Subsequent multi-scenario planning amendment

Static native-error consultation rechecked loading, configuration, packet correlation, dispatch, lifecycle, diagnostics, Close, supervision, and application shutdown.
The resulting inventory separates expected field, connection, producer, HTTP, and whole-application outcomes.
These remain predictions from code, not observed error-test results.

Eight JSONL planning drafts now illustrate the requested flight narrative, focused lifecycle/availability, and separate fault cases.
No parser, scenario engine, or browser was run against them.
The purpose-only catalog and S8 add the explicit multi-scenario request without replacing the existing default/boundary contract.
