## Accepted implementation handoff for Task #91

This comment records the complete accepted design for all four Subtasks. It is part of their execution context.
No chat history, local plan, uncommitted file, or original real-session log is required to understand the assignments.
The issue descriptions define each job. First comments retain detailed contracts, rationale, and prior findings.

### Baseline and authority

* Planning baseline: commit `2a66b7f7730d59cca15ecd381e14c5e21a5ef92b`, including merged #122. Refresh the current tree before implementation.
* Use numeric `baro_setting.pressure_hpa` throughout this work. Completion of #100 is not a prerequisite.
* Use only numeric `baro_setting.pressure_hpa`. Do not migrate legacy `baro_text` into scenarios or add fallback parsing.
* Merged #122 removed wire `schema_version`, shutdown-file support, and diagnostic size options. Do not restore those from older experiments.
* Schema 5 is supported as a named offline diagnostic input, not as a requirement to restore the older publication journal.
* All four Subtasks belong to #91; #91 remains under Epic #9. One developer is the default; execution uses GPT-6.1 Terra at medium.
* Read current AGENTS and contribution instructions, the parent Task, each Subtask's body and first comment, and its required prerequisite results.
* Apply independent review separately from authorship where authorized. Report gaps, decisions, consultations, results, and missing checks in the main implementation conversation.
* Server, telemetry, and embedded Web form one application. Rebuild the complete application after code or build-input changes; no build is required for documentation-only work.
* Preserve one producer, latest-state distribution, independent availability, snapshot-time expiry, source identity, and controlled shutdown.
* No new badge, browser control, provider registry, history queue, interpolation language, recording protocol, or simulator control is selected.
* Issue #91, Epic #9, and related issue #100 had no comments or artifacts at the prepublication read. Current issue metadata and artifact lists must still be refreshed at execution.

### Documentation ownership and staged delivery

These accepted additions were drafted locally and are not assumed to exist on `main` yet.
Subtask 1 must record the selected playback ADR and YAML contract in `docs/adr/0010-yaml-demo-playback.md` and `docs/sources/demo.md`.
Subtask 2 updates current usage and runtime documentation when playback is implemented.
Subtask 3 records the already established mapping result in durable issue context; it does not repeat an open-ended investigation.
Subtask 4 promotes the conversion contract and ADR conversion decision into the same documentation owners with the implemented converter.
Preserve existing subject content and unaffected architecture decisions. Do not overwrite another Subtask's accepted sections.

### Delivery order

* Subtask 1: no prerequisite issue. Migrate YAML data and retain the current default clock temporarily.
* Subtask 2: after Subtask 1. Add typed file selection and the accepted absolute playback clock.
* Subtask 3: no implementation prerequisite; its mapping result is already established during planning.
* Subtask 4: after Subtasks 2 and 3. Implement the selected converter; no additional research or recording format is required.

## Accepted Demo contract snapshot


These requirements define planned YAML playback and recorded-session conversion.
The public API and scenario timing sections describe the current Rust-defined provider.

### Scenario source and default content

The planned provider uses one scenario data source:

* Store the default scenario in the repository as `demo_scenario.yaml`.
* Define scenario values and timing in YAML without duplicating them in Rust.
* Keep parsing, validation, and playback logic in Rust.
* Allow external scenario data to change without recompiling the application. Changes to parsing, validation, or playback logic require a rebuild.
* Preserve every default value, phase, cycle, transition, and unavailable-field case defined in this document.
* Include attitude, coordinated turn, IAS, TAS, GS, speed profiles, references, and IAS acceleration.
* Include altitude, BARO, vertical speed, selected altitude, current heading, and selected heading.
* Use the [current telemetry contract](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/blob/2a66b7f7730d59cca15ecd381e14c5e21a5ef92b/docs/telemetry-contract.md), including numeric BARO, without legacy-format fallbacks.

Independent checkpoints verify default-scenario equivalence. They do not replace the scenario requirements and numeric definitions above.

### Scenario selection

The planned CLI has these selection rules:

| Input or condition | Required behavior |
| --- | --- |
| `--source demo --scenario <path>` | Read the selected scenario |
| Demo without `--scenario` | Parse the embedded repository `demo_scenario.yaml`. Do not search the working directory or executable directory |
| `--scenario` with SimConnect | Reject the combination. Do not add a SimConnect probe or scenario mode |
| Missing, unreadable, or malformed selected file | Report an actionable error without silently selecting another scenario |
| Unsupported scenario format or version | Reject the input without legacy adapters |
| Library invocation | Receive typed source configuration from application composition. Do not read process arguments in the provider |

Preserve the usual Demo startup when no scenario path is supplied.
Scenario delivery follows the [application delivery contract](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/blob/2a66b7f7730d59cca15ecd381e14c5e21a5ef92b/docs/architecture.md#application-delivery-contract).

The selected architecture is recorded in [ADR 0010](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/91).
These selection and format rules are accepted planned behavior. The current provider still uses the Rust API described earlier.

* Resolve an explicit relative path against the caller's working directory. Read and validate the file once before binding HTTP.
* Return a configuration error for an unreadable or invalid scenario. Start neither HTTP nor acquisition after this failure.
* Pass `DemoConfig { scenario: Option<PathBuf> }` through `SourceSelection::Demo(DemoConfig)`.
* Keep `Config` limited to HTTP settings and keep argument parsing in the CLI.
* Rebuild the complete application to change its embedded default.
* Use `--scenario <path>` to edit external data without recompilation. Restart playback to read the changed file.

The planned library interfaces separate preparation from acquisition:

| Interface | Contract |
| --- | --- |
| `DemoConfig::prepare()` | Return `Result<Scenario, DemoError>` after loading and validating the embedded default or explicit file |
| `Scenario::parse(text: &str)` | Return `Result<Scenario, DemoError>` through the same parser and validation used by configuration preparation |
| `demo::run(scenario, publisher, shutdown)` | Consume a validated `Scenario` and Demo `Publisher`. Await shutdown with output `()`. Return `Result<(), DemoError>` |
| `DemoError` | Distinguish file, format, timeline, numeric, and publication failures. Include input identity and field or event location where applicable |

Preparation performs no native acquisition. A failed HTTP bind must still prevent the prepared provider from starting.

### Selected YAML format

The format contains complete events rather than formulas, inherited fields, or interpolated segments.
Its version is independent of the telemetry wire format and diagnostic schemas.

| Field | Accepted value or meaning |
| --- | --- |
| `format` | Required string `msfs-efi-demo` |
| `version` | Required integer `1`. Reject every other version |
| `duration_us` | Required positive integer duration. Every event occurs before this boundary |
| `end` | Required `repeat` or `wait` |
| `reference` | Optional default-helper metadata. Its only supported member is `attitude_cycle_samples` |
| `reference.attitude_cycle_samples` | Positive integer no greater than the event count. The embedded default uses 280 |
| `events` | Required nonempty sequence of complete events |
| `events[].at_us` | Nonnegative integer offset from the cycle origin. Nondecreasing order preserves equal-time events in file order |
| `events[].state` | One of the current telemetry states: `live`, `waiting`, `paused`, `stale`, `disconnected`, or `invalid` |
| `events[].sample_age_us` | Required only for `live`. Nonnegative acquisition age at the scheduled event time |
| `events[].sample` | Required only for `live`. Contains required valid `attitude` and independently optional current sample fields |

Event samples use the [telemetry field names, units, and numeric bounds](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/blob/2a66b7f7730d59cca15ecd381e14c5e21a5ef92b/docs/telemetry-contract.md#fields-and-conventions).
They exclude transport fields such as `source`, `sequence`, and `sample_age_ms`.

* A missing or null optional value means unavailable in that event. Do not inherit previous values or fill Demo defaults.
* Validate present numbers independently. Reject invalid present values instead of clamping them or converting them to apparent readings.
* Preserve partial speed profiles. Each threshold, reference, and redline remains independently optional.
* Use only numeric `baro_setting.pressure_hpa`. Reject legacy BARO fields.
* Non-live events contain neither `sample` nor `sample_age_us`. They preserve the last accepted acquisition time.
* Reject unknown fields, duplicate keys, multiple YAML documents, fractional timestamps, and unsupported tags or inclusion directives.
* Reject timeline arithmetic that cannot be represented by the playback clock. Apply documented finite input and parser budgets.
* YAML anchors may reuse complete values. Reject YAML merge keys (`<<`) instead of introducing field inheritance.

This fictional example demonstrates a gap and finite completion. It is not the default scenario:

```yaml
format: msfs-efi-demo
version: 1
duration_us: 2000000
end: wait
events:
  - at_us: 0
    state: live
    sample_age_us: 0
    sample:
      attitude: { pitch_deg: 0.0, roll_deg: 0.0 }
      baro_setting: { pressure_hpa: 1013.25 }
  - at_us: 1500000
    state: paused
```

The embedded default contains 3960 events at 50000 Âµs intervals, starting at zero.
Its last event occurs at 197950000 Âµs. Its duration is 198000000 Âµs and its end policy is `repeat`.

* Store every default value and optional omission in YAML, including pressure in hPa and separately supplied acceleration and VSI.
* Preserve the six inclusive altitude sweeps, target phases, north crossings, and shorter cycles through independent checkpoints.
* Keep `sample(index)`, `attitude(index)`, and `speed_profile()` as views of the parsed embedded data.
* Derive the attitude helper's repeat from `reference.attitude_cycle_samples` and reuse the default's first seven poses.
* Custom-file playback uses its prepared scenario directly. It does not call helpers that select the embedded default.

### Planned clock and completion

Absolute monotonic deadlines replace the current sample-index delay policy when YAML playback is implemented.
Late execution no longer stretches recorded gaps. Nominal default checkpoints and timing remain unchanged.

| Condition | Planned behavior |
| --- | --- |
| Before the first event | Keep `waiting` and run independent freshness checks |
| Live event | Set acquisition time to the scheduled event instant minus `sample_age_us`. Publish the complete sample through `publish_sample_at` |
| Repeated equal fresh samples | Process both events. Zero acquisition age at each distinct deadline renews freshness |
| Optional-only update | Use an age that preserves the previous acquisition instant. Do not make cached attitude fresh |
| No events during a gap | Publish no cached sample. Continue independent 50 ms expiry checks |
| Late event | Preserve its intended acquisition instant. Snapshot reads suppress already expired values before delivery |
| Several due events | Process in file order with bounded work between shutdown and expiry checks. Subscribers may skip intermediate publications |
| `repeat` at `duration_us` | Advance the origin by the declared duration. Continue sequence numbering and process the next cycle's events |
| `wait` at `duration_us` | Stop scenario events but retain the provider and expiry checks until shutdown. Last live values expire normally |
| Shutdown | Prioritize shutdown over scenario events and expiry checks |

Preserve snapshot-time age checks as well as `Publisher::expire()`.
Publishing an old event with its intended timestamp must never expose expired data as live, even before the next expiry tick.

For each event, the intended acquisition time must not move backward relative to earlier live events.
Repeat boundaries must preserve this rule and must not reuse an earlier cycle's cached acquisition as a fresh sample.

### Playback semantics

Playback preserves the shared runtime boundaries:

* Publish scenario data through the existing `Publisher`, HTTP/SSE, and EFI path.
* Use one producer shared by all clients and the existing latest-state distribution.
* Make playback deterministic, with explicit repeat and end behavior.
* Validate the timeline and numeric values before relying on them for publication.
* Preserve pauses, data gaps, and missing or unavailable indications.
* Apply the [freshness rules](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/blob/2a66b7f7730d59cca15ecd381e14c5e21a5ef92b/docs/telemetry-contract.md#age-calculation). Replaying cached data must not make it appear newly live.
* Preserve controlled shutdown and the [provider responsibilities](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/blob/2a66b7f7730d59cca15ecd381e14c5e21a5ef92b/docs/sources/index.md#shared-responsibilities).
* Identify replay as `Source::Demo`. Recorded data does not represent a current live flight.
* Require no simulator, SDK, or SimConnect DLL for playback.

### Recorded-session conversion

Conversion must produce the same scenario format consumed by Demo:

* Provide a documented conversion procedure for recorded SimConnect logs.
* Select one session explicitly when a log contains appended sessions.
* Preserve event timing, data gaps, and missing or unavailable values.
* Exclude diagnostic-only records and local environment metadata from scenario data.
* Keep generated real-session logs and local SDK paths outside commits.
* Interpret log records using the [diagnostic schema rules](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/blob/2a66b7f7730d59cca15ecd381e14c5e21a5ef92b/docs/sources/simconnect.md#diagnostic-schema-interpretation).

The selected conversion contract is accepted planned behavior under [ADR 0010](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/91).
It supports diagnostic schemas 4 and 5 through the following explicit mappings.
No production converter is implemented yet.

### Conversion interface and input

The planned Rust executable has this invocation:

```powershell
demo-convert --input <jsonl-path> --session <session-id> --output <yaml-path>
```

* Put the thin executable in `src/bin/demo-convert.rs`. Keep conversion logic in the library.
* Expose `demo::convert_session(reader: impl BufRead, session_id: &str) -> Result<Scenario, ConversionError>`.
* Add `Scenario::write_yaml(writer: impl Write) -> Result<(), DemoError>` using the same scenario representation and validator.
* Resolve relative input and output paths against the caller's working directory.
* Require an exact session identifier, even when the input contains only one session. Never select the latest session implicitly.
* Require one selected `session_start`, contiguous record numbers starting at zero, monotonic integer `elapsed_us`, and a selected `session_end`.
* Parse `record` as an unsigned JSON integer. Accept `elapsed_us` as an unsigned decimal string or unsigned JSON integer without rounding.
* Require diagnostic schema 4 or 5 and `live_publication_enabled: true`. Reject schemas 1-3, unsupported versions, and untimed raw SSE files.
* Preserve equal timestamps in record order. Reject duplicate selected session identifiers, interleaved selected records, and malformed records within the selected session.
* Allow an interrupted fragment outside the selected session, separated by a later valid `session_start`. Report that exclusion.
* Reject an incomplete selected session. Do not invent its missing duration or silently convert a prefix.
* Accept a completed session with a reported source failure if it contains replayable records. Report the failure outside scenario data.
* Reject a session with no replayable records. Do not invent a live sample or substitute the default scenario.
* Validate and serialize before creating the output. Require an existing parent directory and create the output exclusively without overwriting.
* Remove only a newly created output if its write fails. Return an actionable path, session, line, or record error as applicable.

Other sessions contribute neither state nor samples to the selected session.
Metadata from unselected sessions must not affect schema selection.
Ignore malformed fragments outside the selected session only when session boundaries make their exclusion unambiguous.

### Conversion time and freshness

| Value | Selected meaning |
| --- | --- |
| Time origin | The selected session's monotonic zero. Retain `elapsed_us` offsets without subtracting the first sample time |
| Scenario end | `duration_us = session_end.elapsed_us + 1`, with checked arithmetic and `end: wait` |
| Event timestamp | The selected diagnostic record's `elapsed_us`, or a deterministic expiry deadline in schema 4 |
| Attitude acquisition for replay | The timestamp of an eligible normalized `attitude_sample` observation |
| Live `sample_age_us` | Event timestamp minus the retained attitude-observation timestamp |
| Equal fresh attitudes | A later eligible observation advances the acquisition timestamp even when angles are equal |
| Optional-only updates | Retain the preceding eligible attitude-observation timestamp |
| Non-live events | Preserve the preceding accepted acquisition time through the existing Publisher contract |

Journal timestamps follow native receipt. They identify recorded observations, not exact packet acquisition instants.
Replay uses those observation times consistently and does not renew cached attitude on optional updates.
Do not subtract rounded `sample_age_ms` from a later journal timestamp to manufacture an exact acquisition time.

### Schema 4 observation mapping

Schema 4 converts source observations into deterministic replay events.
It does not reconstruct original publication count, sequence, batching, or exact native receipt times.

Reuse the pure `Lifecycle` and `Instruments` rules in an internal `simconnect::recorded` module.
Use a checked synthetic `Instant` origin plus recorded offsets. Conversion must not load or call SimConnect.

| Record or input | Mapping |
| --- | --- |
| `api_result` for successful `SimConnect_Open` | Start a new connection generation, connect lifecycle, and clear retained samples/instruments |
| `dispatch.message.Event`, ID 10 | Apply `sim(data != 0)` |
| `dispatch.message.Event`, ID 11 | Apply `pause(data)` |
| `dispatch.message.Filename`, ID 12 | Clear instruments and apply a flight transition. Keep the filename only in temporary conversion state |
| `dispatch.message.SystemState`, request 20 | Apply the recorded Sim-running value |
| `dispatch.message.SystemState`, request 21 | Update flight identity. Clear instruments if the flight changes |
| `attitude_sample` | Require `candidate_is_hypothesis: false`. Apply validated normalized attitude or the recorded validation failure |
| `instrument_sample` | Validate index/name and number classification. Apply only while both recorded `active` and projected lifecycle are active |
| `instrument_request` with an error; `instrument_unavailable` | Remove that independently optional field and its dependent indications |
| `connection_lost` | Disconnect lifecycle and clear instruments and attitude |
| `state_transition.published` | Preserve recorded unavailable-state observations and compare its complete snapshot with the projection |
| `attitude_raw`, degree probes, and repeated-radian checks | Diagnostic evidence only. Do not generate another sample or use them to override normalized attitude |
| `dispatch` attitude/instrument payloads | Do not apply again. Their normalized observation records supply the conversion inputs |
| Other established diagnostic records | Exclude from scenario data. Native errors alone do not imply a disconnect |

* Reject inconsistent instrument index/name pairs and unknown semantic record shapes instead of guessing their meaning.
* Decode `classification: finite` from its finite numeric value. Treat `nan` and signed infinities as unavailable observations.
* Do not decode bit patterns to restore an unavailable value. Ignore display text and diagnostic probes.
* Clear instruments whenever lifecycle becomes inactive. Stale or invalid attitude alone does not clear independently fresh optional observations.
* Evaluate lifecycle and expire instruments at each semantic record and each intervening known expiry deadline.
* Process expiry before a record at the same timestamp. Preserve record order for subsequent equal-time observations.
* Emit complete live samples for eligible new attitudes and changes or expiry of active optional data.
* Emit unavailable events when state changes. Preserve explicit recorded unavailable checkpoints without renewing acquisition age.
* After an unavailable checkpoint, require a new eligible normalized attitude before restoring live data.
* Keep a separate live-eligibility gate. While closed, retain the latest unavailable state if the pure projection would otherwise appear live.
* Reopen that gate only for a subsequent valid normalized attitude observed while projected lifecycle is active.
* Stop synthetic deadlines at the selected session's recorded end. Add no periodic live resampling.

Expose an internal next-expiry query on the existing pure types if needed.
Keep expiry constants and estimator calculations with their current owners rather than duplicating them in the converter.

State-transition snapshots provide comparison evidence, not permission to retime observations silently.
Report timing-sensitive checkpoint differences outside YAML without asserting an unproven cause.
Keep recorded unavailable states authoritative. Reject structural contradictions or missing required semantic records.
Expiry-boundary or estimator differences can be timing-sensitive. Live data after an explicit unresolved pause is a structural contradiction.

### Raw optional-instrument mapping

Schema 4 reuses `Instruments::sample` for these conversions and availability gates:

| Index | Raw observation | Scenario field or rule |
| --- | --- | --- |
| 0, 1, 2 | IAS, TAS, GS in knots | `ias_kt`, `tas_kt`, `gs_kt` |
| 3 | Indicated altitude in feet | `altitude_ft` |
| 4 | KOHLSMAN setting in inHg | Numeric `baro_setting.pressure_hpa`, using `HPA_PER_INHG` |
| 5 | Vertical speed in ft/min | `vertical_speed_fpm`; do not derive it from altitude |
| 6 | Magnetic heading in degrees | Normalize into `[0, 360)` for `heading_deg` |
| 7 | Ball position | Divide by 127 and validate `slip_skid` |
| 8 | Turn rate in degrees/second | `turn_rate_dps` |
| 9, 10 | VS0 and VS1 in knots | Independent profile members `vs0_kt`, `vs1_kt` |
| 11 | Minimum rotation speed in knots | `airspeed_profile.references.vr_kt` |
| 12 | Current maximum speed in knots | `airspeed_profile.redline_kt`; do not substitute fixed VNE |
| 13 | Autopilot available | Targets require the value to equal 1 |
| 14 | Selected heading in degrees | Normalize and expose only with available autopilot |
| 15 | Selected altitude in feet | Expose only with available autopilot and index 16 equal to 0 |
| 16 | Radio-altitude reference mode | Value 0 permits the barometric altitude target; other values suppress it |

Optional observations expire independently after 1000 ms.
IAS acceleration uses the existing 500 ms acquisition window, 256-entry history bound, and 500 ms silence expiry.
Preserve invalid-value removal, independent profile members, target gates, and reset behavior.
Do not invent VFE, VNO, VNE, extra reference speeds, or another acceleration estimator.

### Schema 5 publication mapping

Schema 5 converts each actual `publication.data.snapshot` at the publication record's `elapsed_us`.
Its timing still describes the recorded publication observation rather than exact native receipt.

* Preserve every publication's state and complete supported sample fields, including missing independently optional values.
* For live publications, require the latest preceding valid normalized attitude observation in the same connection generation.
* Track control records through the same pure lifecycle rules as schema 4, without emitting additional control events.
* Accept attitude candidates only while lifecycle is active. Clear eligibility on pause/resume, Sim changes, flight transitions, disconnect, and connection reset.
* Require its angles to match the publication. Reject a live publication without that matching observation.
* Use its observation timestamp to calculate `sample_age_us`. Reuse it for optional-only publications, including equal-angle cached updates.
* Clear candidate eligibility after unavailable publications and connection changes. Observations made during a pause cannot authorize later recovery.
* Do not add another event for `state_transition`, `dispatch`, or optional observations already represented by publication snapshots.
* Copy numeric BARO only. Discard `baro_text` without parsing it or using it as a fallback.
* Drop source sequence, source identity, wire version, and reported age from scenario samples. Playback supplies its own Demo identity and sequence.
* Compare reported `sample_age_ms` only as evidence. Do not reinterpret it as an exact acquisition timestamp.

Known diagnostic-only kinds can be ignored. Reject unknown replay-bearing records or unsupported snapshot fields instead of silently dropping indications.
Validate the completed events through the common scenario validator before writing YAML.

### Conversion report and evidence limits

Report the selected schema, event count, excluded fragments, source failure, and checkpoint differences outside the YAML file.
Keep session identifiers, file identities, paths, process data, DLL information, and environment details out of scenario data.
The converter must not read `validation_scope` to select behavior.

Schema 4 establishes observation projection; schema 5 preserves recorded publication payloads at journal timestamps.
Neither mapping establishes exact original native timing, original browser paint, physical acceleration truth, or live/device acceptance.


## Architecture decision snapshot

### ADR 0010: YAML Demo playback with separate session conversion

Decision status: Accepted for scenario selection, playback, and recorded-session conversion.

## Context

[Task #91](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/91) requires editable YAML scenarios and replay of recorded SimConnect sessions.
The current Demo provider stores scenario values, phases, and formulas in Rust.
Its default cycle contains 3960 samples at nominal 20 Hz.

The selected design preserves the [application composition](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/blob/2a66b7f7730d59cca15ecd381e14c5e21a5ef92b/docs/adr/0007-application-composition.md) and [SSE lifecycle](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/blob/2a66b7f7730d59cca15ecd381e14c5e21a5ef92b/docs/adr/0002-sse-connection-lifecycle.md).
The [Demo contract](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/91) owns the planned format and playback rules.
Use the accepted numeric BARO contract. Completion of #100 is not an implementation prerequisite.

## Decision

| Boundary | Selected direction | Rationale |
| --- | --- | --- |
| Scenario representation | Complete timed telemetry events with explicit format, version, duration, and end policy | Represents default samples and recorded availability without an interpolation language |
| Default delivery | Embed the repository's `demo_scenario.yaml` and parse it through the external-file reader | Preserves single-executable startup outside the repository |
| Custom selection | Read an explicit path once during configuration preparation | Allows external data edits without rebuilding and reports errors before startup |
| Application interface | Pass `DemoConfig` through `SourceSelection::Demo(DemoConfig)` | Keeps process arguments in the CLI and HTTP settings in `Config` |
| Playback clock | Use absolute monotonic event deadlines and separate acquisition age | Preserves recorded gaps and prevents delayed events from appearing newly acquired |
| Runtime ownership | Retain one provider, one active `Publisher`, independent expiry, and latest-state subscriptions | Keeps acquisition independent of clients and HTTP availability |
| Conversion | Keep conversion offline and produce the same YAML format | Avoids a second runtime reader or SimConnect dependency for replay |

The no-path default changes only after rebuilding the complete application.
An explicit file changes playback on the next launch without recompilation.

### Parsing and validation

* Select `serde-saphyr` for typed Serde parsing. Verify Windows/MSVC and toolchain compatibility during implementation.
* Reject unsupported format identifiers, versions, unknown fields, duplicate keys, invalid timelines, and invalid numeric data.
* Use explicit parsing budgets. Enable no file inclusion, environment substitution, custom object construction, or network resolver.
* Preserve partial profiles and independent optional values under [ADR 0008](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/blob/2a66b7f7730d59cca15ecd381e14c5e21a5ef92b/docs/adr/0008-partial-airspeed-profile.md).
* Reject invalid present values. Missing or null optional fields remain unavailable without inheritance.

The [parser documentation](https://docs.rs/serde-saphyr/2.1.1/serde_saphyr/) describes typed parsing, duplicate-key rejection, configurable budgets, and Rust 1.89 or newer.
That documentation supports dependency selection. It does not establish compatibility with this application's Windows build.

### Scope of the timing change

The current provider advances its scenario only when it publishes a sample.
Late ticks therefore stretch the scenario.
The selected absolute timeline replaces that policy when YAML playback is implemented.
Nominal default values, boundaries, and the 198-second cycle remain unchanged.

* Keep intended acquisition time when an event is late. Never replace it with the current publication time.
* Preserve the snapshot-time expiry guard and the independent freshness tick.
* Process due events in file order with bounded work between shutdown checks. The latest-state channel may coalesce intermediate publications.
* At a finite scenario's end, keep the provider alive until shutdown. Do not trigger failed-producer recovery by returning normally.

### Existing default helpers

Preserve `sample`, `attitude`, and `speed_profile` as views of parsed embedded scenario data.
Store the standalone attitude reference cycle in YAML metadata and reuse the default's initial seven poses.
Do not retain Rust copies of poses, profile values, cycle lengths, or scenario formulas.
This preserves the separately documented helper behavior without introducing multiple playback timelines.

## Alternatives and trade-offs

| Option | Benefit | Cost or failure | Result |
| --- | --- | --- | --- |
| Retain hardcoded Rust scenarios | Small current implementation | Requires rebuilding for scenario changes and does not meet YAML ownership | Reject |
| Complete timed events | Direct values, explicit omissions, simple independent checkpoints, one playback representation | Larger default YAML and less compact editing | Select |
| Generic segments or expression language | Compact sweeps, phases, and repeated cycles | Adds interpolation, inheritance, slope, angular-wrap, and boundary semantics | Defer until an actual authoring need justifies the extra language |
| Required file beside the executable | Editable no-path default | Adds a deployment artifact and missing-file startup failure | Reject for the no-path default |
| Working-directory search with fallback | Convenient in some checkouts | Makes selection depend on launch location and can hide an explicit failure | Reject |
| Sample-index pacing for all scenarios | Preserves current stalled-runtime progression | Stretches recorded pauses and gaps | Replace with absolute timing |
| Runtime JSONL or SimConnect replay source | Reads logs directly | Adds another runtime contract and couples Demo to diagnostic interpretation | Keep offline conversion |

## Conversion evidence and decision

The conversion mapping was resolved on 2026-11-06 within Task #91's accepted scope.
This decision replaces the earlier unresolved comparison. It changes neither the selected YAML format nor the recording protocol.
The [Demo conversion contract](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/91) owns invocation, record mapping, numeric rules, and failure behavior.

Remote revision [2a66b7f](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/commit/2a66b7f7730d59cca15ecd381e14c5e21a5ef92b) uses schema 4 diagnostics and untimed raw SSE recording.
Available schema 5 diagnostic recordings contain actual publication snapshots.

| Input | Decision | Evidence limit |
| --- | --- | --- |
| Diagnostic schema 4 | Support deterministic observation projection through existing pure lifecycle and instrument rules | Does not reconstruct original publication count, sequence, batching, or exact native receipt time |
| Diagnostic schema 5 | Support publication payloads at their recorded timestamps, with matching normalized attitude observations | Journal timestamps follow acquisition; rounded snapshot age does not recover exact acquisition time |
| Diagnostic schemas 1-3 | Reject | Candidate live states do not establish actual live publication. Schema 2 could invalidate cross-unit disagreement |
| Untimed raw SSE JSONL | Reject | Missing event timestamps prevent preserving gaps. Latest-state delivery can skip publications |

Select a thin Rust `demo-convert` executable with required input, exact session identifier, and output arguments.
Require a complete selected session. Reject ambiguous selection, unsupported shapes, interrupted selected data, and existing output files.
Support historical diagnostic schemas only through these named offline mappings; add no runtime compatibility path or legacy BARO parser.

### Ownership and time

* Keep native loading and recording unchanged. Place the internal projection beside the existing pure `Lifecycle` and `Instruments` types.
* Reuse normalization, optional expiry, target gates, and the accepted IAS estimator. Add only the internal access needed for deterministic deadline processing.
* Let a normalized attitude observation establish the replay acquisition timestamp. Optional-only updates retain that timestamp.
* Preserve source-observation timestamps for schema 4 and publication-record timestamps for schema 5.
* Preserve known unavailable observations, gaps, independent missing fields, and session-relative timing without periodic live resampling.
* Use the existing scenario validator and one YAML writer. Keep diagnostic metadata outside the output.

This clock preserves freshness within recorded observation time.
It does not claim exact packet receipt or reproduce every historical browser frame.
The limitation follows from recorded evidence and does not prevent the selected conversion contract from being implemented.

### Conversion alternatives

| Option | Assessment |
| --- | --- |
| Keep conversion unresolved until new recordings exist | Unnecessary: current observation records and available publication logs support explicit bounded mappings |
| Support only schema 4 | Serves current recordings but discards available schema 5 publication evidence |
| Support only schema 5 | Uses available recordings but cannot consume the current diagnostic output |
| Add a new publication journal or timestamped SSE wrapper | Changes recording behavior without a need for the selected observation-replay outcome |
| Duplicate lifecycle, units, and estimator calculations in a script | Risks drift from accepted instrument behavior |
| Reuse pure Rust rules and accept schemas 4 and 5 explicitly | Select: covers current and available evidence without loading SimConnect or inventing missing timestamps |

No further investigation is an implementation prerequisite for these mappings.
Required validation includes independent synthetic schema-4 traces, schema-5 record mapping, failure paths, and converted-file playback through the application.
The accepted design and focused mapping experiments do not establish a completed Rust converter or simulator/device acceptance.

## Consequences and validation

* Scenario data becomes an application input. Explicit-file parsing failures must name the path and the invalid field or timeline location.
* The default and custom files use one parser and one playback engine. No invalid explicit file selects another scenario.
* Replay uses `Source::Demo` and never loads SimConnect. It adds no visible source badge or browser control.
* Larger YAML data replaces scenario-specific arithmetic in production Rust. Independent fixtures remain the acceptance oracle.
* The implementation must preserve numeric BARO, optional-field replacement, continuous publication sequence, age, and controlled shutdown.
* Follow the [validation matrix](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/blob/2a66b7f7730d59cca15ecd381e14c5e21a5ef92b/docs/testing/index.md#validation-by-change-scope), [Demo checks](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/blob/2a66b7f7730d59cca15ecd381e14c5e21a5ef92b/docs/testing/runtime.md#telemetry-and-demo-checks), and Task #91's additional browser requirements.
* Use controlled clocks for late events, equal fresh samples, gaps, repeats, finite completion, and shutdown.
* Verify real HTTP/SSE and embedded EFI with default and custom scenarios. Inspect the required screenshots.
* Application implementation, converter fidelity, simulator compatibility, and phone acceptance remain separate from this decision's acceptance.

## Reconsideration

Reconsider event expansion if measured file size, startup cost, or repeated editing makes full samples unsuitable.
Reconsider the embedded default if installation requirements explicitly require an editable no-path default.
Reconsider conversion if a required recording lacks the selected semantic records or a new fidelity requirement needs exact native-publication history.
Any recording extension must preserve the existing acquisition and diagnostic failure boundaries.

## Required application validation

Run the following complete application checks on Windows x64 MSVC after implementation:


```powershell
$env:RUSTUP_TOOLCHAIN = 'stable'
$env:CARGO_BUILD_TARGET = 'x86_64-pc-windows-msvc'
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
node --test tests/*.test.mjs
cargo build --locked --release
```

Run both desktop scripts against the rebuilt release application:

```powershell
node tests/browser/pfd-browser.mjs
node tests/browser/pfd-runtime-browser.mjs target/x86_64-pc-windows-msvc/release/msfs2024-artificial-horizon.exe
```

* Configure Playwright and the browser through the [desktop procedure](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/blob/2a66b7f7730d59cca15ecd381e14c5e21a5ef92b/docs/testing/pfd.md#responsive-efi-layout-and-desktop-checks).
* Subtask 1 checks the YAML-backed default. Subtask 2 adds production-path coverage for `--scenario`. Subtask 4 checks converted input.
* Do not infer custom-file or converted-file integration from default-only scripts or injected browser fixtures.
* Use paused Tokio clocks for timeline and age assertions. Keep expected values independent of the new parser and YAML under test.
* Compare matched baseline/candidate screenshots for affected scenes. Record causes only where evidence supports them.
* Select meaningful negative controls where an assertion could pass despite using the wrong file, current-time stamping, or an override.
* Obtain a separate review of each exact integrated code diff under [project-review-change](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/blob/2a66b7f7730d59cca15ecd381e14c5e21a5ef92b/.agents/skills/project-review-change/SKILL.md). Record missing authorization or capability explicitly.
* Repeat affected checks after corrections. Run the complete applicable suite and review after each implementation outcome.
* Verify the combined application and converter before Task completion. Reuse earlier evidence only under the documented evidence-reuse rules.
* Preserve established mobile and instrument-value acceptance. Desktop replay checks do not establish new live-simulator or real-device acceptance.
* Planning experiments do not replace these implementation checks. Record unperformed checks explicitly.

