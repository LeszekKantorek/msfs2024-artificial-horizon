## Outcome

Replace the Rust-hardcoded Demo scenario with deterministic playback of a repository-owned YAML scenario. Run a chosen scenario with:

```text
--source demo --scenario <path>
```

Recorded real SimConnect sessions can be used to prepare replayable demo scenarios.

## Requirements

- Add `demo_scenario.yaml` to the repository. It must represent every value, timing rule, phase, cycle, transition and unavailable-field case currently hardcoded in the Rust Demo provider, including attitude, coordinated turn, IAS/TAS/GS, speed profile/references, IAS acceleration, altitude/BARO, VSI, selected altitude and current/selected heading.
- Demo reads the scenario and plays it through the existing Publisher, HTTP/SSE and EFI pipeline. Remove the duplicated hardcoded scenario content from Rust; retain only generic parsing, validation and playback logic.
- Support `--scenario <path>` for `--source demo`. Without an explicit path, use the checked-in default scenario through a documented deployment-safe default; preserve the usual Demo startup.
- Provide a documented way to turn a selected recorded SimConnect log session into the same YAML scenario format. Select one session explicitly from appended logs, preserve event timing and missing/unavailable values, and exclude diagnostic-only records and local environment metadata from scenario data.
- Replayed sessions are clearly identified as Demo. They do not load SimConnect or represent a current live flight.
- Preserve one producer shared by clients, latest-state distribution, freshness semantics and controlled shutdown. Replaying a pause or data gap must not make cached samples look newly live.

## Implementation baseline and findings

This issue is self-contained and does not depend on local review notes or conversation history.
The implementation observations below refer to revision `122b9ffffc034953514a60efe8071d212a410ee8`, reviewed with PR #98.
Recheck the current implementation baseline before edits.

| Area | Observed implementation | Required work |
| --- | --- | --- |
| Scenario content | `src/providers/demo.rs` contains values and timing. `demo_scenario.yaml` is absent | Move scenario content to YAML and retain generic playback logic |
| Source configuration | `src/application.rs` uses parameterless `SourceSelection::Demo` | Pass typed Demo configuration through application composition |
| CLI | `src/bin/main.rs` and its test reject `--scenario` | Support the option for Demo and reject it with SimConnect |
| BARO | Producers, APIs, and fixtures still contain `baro_text` | Use only numeric `baro_setting.pressure_hpa` in the affected code and fixtures |
| Timing | Separate sample and freshness intervals run every 50 ms and skip missed ticks | Preserve freshness independently of sample publication and define replay scheduling |
| Default progression | Published sample index advances modulo 3960. Skipped ticks do not advance it by elapsed wall time | Preserve nominal default checkpoints and explicitly specify late-tick behavior |

The default full cycle contains 3960 samples at nominal 20 Hz, lasting 198 seconds.
The feature documentation owns the individual values, phases, and shorter cycles.
Do not copy obsolete implementation representations into YAML solely to reproduce existing fixtures.

### Established constraints

* Scenario files are runtime input data. Editing an explicit scenario requires no Rust source change or recompilation.
* Changes to application code or its supported contract require a coordinated rebuild of server, telemetry, and embedded browser.
* Use only numeric `baro_setting.pressure_hpa`. Missing or invalid pressure remains unavailable without `baro_text` fallback.
* Preserve independent instrument availability, one shared producer, latest-state delivery, freshness, and controlled shutdown.
* Preserve accepted instrument values. Missing separate device records do not reopen their acceptance.
* Use independent expected checkpoints. Do not derive expected results from the new parser or scenario under test.
* Replay uses `Source::Demo` and never loads SimConnect. A visible source badge is not required by this issue.
* Reject unsupported scenario formats or versions without maintaining legacy parsers or silent scenario substitution.

### Planning choices within this Work Item

Resolve these details during implementation planning. The alternatives are not additional approved product behaviors.

| Topic | Required planning result |
| --- | --- |
| Default delivery | Select embedding or a deployed file. Define path resolution and actionable failure behavior for deployment outside the repository |
| YAML structure | Specify timed samples or segments, fields, numeric validation, timestamp ordering, and repeat/end semantics |
| Format identification | Define supported-format identification and unsupported-version rejection without backward-compatibility adapters |
| Late ticks | Define pacing or sample skipping. Distinguish nominal default playback from recorded event timing and gaps |
| Conversion interface | Choose a documented procedure or tool, its location, explicit session selection, supported inputs, and output validation |
| Record mapping | Define which records supply observations, actual publications, lifecycle transitions, and unavailable values |
| Source identity | Verify Demo identity through the source/telemetry contract without introducing an unrequested browser badge |

### Recorded-data evidence boundaries

Apply these limits when selecting and converting diagnostic records:

* Schema 1–3 candidate live states do not establish actual live telemetry publication.
* Schema 2 cross-unit disagreement could invalidate samples. Account for this when interpreting recorded unavailable states.
* Schema 4 identifies enabled live publication. Schema identity alone does not prove integrated simulator or mobile acceptance.
* Do not infer publication semantics from a field name containing `candidate` alone.
* Select supported input schemas explicitly. Reject unsupported inputs rather than silently treating them as current records.
* Exclude diagnostic-only records and local environment metadata from scenario data.
* Preserve recorded timing, pauses, gaps, and independent missing values. Do not refresh cached values through replay ticks.
* Conversion must not depend on `validation_scope`, which #100 removes from diagnostic output.
* Replay verifies playback behavior. It does not establish physical acceleration truth, original painted browser frames, or device acceptance.
* Do not treat receipt timing as a universally confirmed cause of IAS spikes. This Work Item does not change the accepted estimator or prediction.

## Possible approach

Use a versioned YAML format describing timed samples or generic timed segments, with explicit repeat/end behavior and optional fields. First inventory the existing Rust Demo behavior and prove equivalent output from the default file against independent existing checkpoints.

A local conversion tool can read one real JSONL session, normalize source observations and lifecycle events, and write this same YAML format. Keep conversion separate from playback so the runtime has one scenario reader. The format and conversion details should be established during implementation planning; this section describes an approach, not a prescribed implementation.

## Acceptance criteria

- [ ] `demo_scenario.yaml` is committed and contains all current Demo scenario values and timing behavior; Rust no longer contains a second copy of that content.
- [ ] Default Demo and `--source demo --scenario <path>` both deliver the scenario through real HTTP/SSE to EFI.
- [ ] Existing independent Demo checkpoints continue to pass for the migrated default file, including cycles, boundary samples, optional fields and unavailable cases.
- [ ] A recorded SimConnect session can be converted to YAML and replayed with its timing and data availability preserved; session selection and conversion are documented.
- [ ] Explicit scenario changes appear in playback without Rust source edits or recompilation.
- [ ] Missing/unreadable/malformed scenarios and unsupported versions produce actionable errors without silently selecting another scenario; validate CLI combinations, timeline and numeric values.
- [ ] Timing, repeat/end behavior, pause/gap handling, equal fresh samples, multiple clients and shutdown have deterministic automated coverage.
- [ ] The normal application checks and affected browser runtime checks pass; documentation explains the default file, custom paths, conversion and Demo identification.
- [ ] No simulator, SDK or DLL is required for YAML playback. Generated real-session logs and local SDK paths are excluded from commits.

* [ ] YAML, conversion output, and fixtures use numeric BARO without legacy text fields or fallback.
* [ ] Typed library configuration supports scenario selection without reading process arguments inside the provider.
* [ ] Default selection works from a deployed location outside the repository. Explicit-file failures never select another scenario.
* [ ] Late-tick, repeat/end, and gap semantics are documented and verified using a controlled clock.
* [ ] Conversion tests distinguish supported diagnostic schemas, selected sessions, candidate observations, publications, and unavailable states.
* [ ] Conversion excludes diagnostic metadata and does not require `validation_scope`.
* [ ] Replay retains Demo source identity without adding a browser badge or loading SimConnect.
* [ ] Validation records identify the tested artifacts and evidence limits without relying on local review notes.

## Validation additions

Run the current Windows x64 MSVC application checks, Node tests, and complete release build.
Run both desktop browser scripts with the rebuilt release executable.
Use controlled clocks for scheduling, gaps, repeated equal samples, and shutdown.
Use fixtures with independent expectations for conversion and default-scenario equivalence.

Record the tested revision, commands, environment, selected session, diagnostic schema, and conversion result.
Retain artifact identities and missing checks in this issue or a durable linked validation record.
Do not commit generated real-session logs or environment metadata.
Automated playback tests remain independent of the simulator, SDK, and DLL.
Desktop results do not certify real-device or live-simulator behavior.

## Coordination

This Work Item can start independently of #100.
Use the accepted numeric `baro_setting.pressure_hpa` contract in scenarios, conversion output, fixtures, and affected APIs.
Update affected code and tests within this Task's scope. Do not migrate obsolete BARO text fields or add legacy fallbacks.
Planning and implementation do not require completion of #100.

Follow-up to the Demo and SimConnect integration under #9. The new Demo-only `--scenario` option intentionally replaces the previous rejection of that option; it does not add a SimConnect probe/scenario mode. Keep source configuration in the application/library boundary rather than reading process arguments inside the library.

## Reference documentation

The following pinned documents preserve the requirements reviewed with PR #98:

| Subject | Source |
| --- | --- |
| Default values, cycles, and planned playback | [Demo provider](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/blob/122b9ffffc034953514a60efe8071d212a410ee8/docs/sources/demo.md) |
| Diagnostic record interpretation | [SimConnect diagnostic schemas](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/blob/122b9ffffc034953514a60efe8071d212a410ee8/docs/sources/simconnect.md#diagnostic-schema-interpretation) |
| Numeric data, availability, and freshness | [Telemetry contract](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/blob/122b9ffffc034953514a60efe8071d212a410ee8/docs/telemetry-contract.md) |
| Required validation | [Testing guide](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/blob/122b9ffffc034953514a60efe8071d212a410ee8/docs/testing/index.md) |

## Execution handoff

The [first comment with the accepted implementation contract](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/91#issuecomment-6696677478) preserves the complete YAML format, runtime semantics, conversion mappings, rationale, and validation.
The descriptions of the following Subtasks define their outcomes and acceptance. Each first comment supplies additional detailed context.
No chat history, local plan, uncommitted file, or original real-session recording is required to understand the assignments.

| Subtask | Outcome | Prerequisites |
| --- | --- | --- |
| #123 | Implement the YAML format and migrate the default scenario | None |
| #124 | Integrate scenario playback and selection through EFI | #123 |
| #125 | Establish and record conversion evidence and mapping | None; mapping result established during planning |
| #126 | Convert a selected recorded session and replay it | #124 and #125 |

The selected mappings accept complete schema 4 diagnostic sessions as observation projection and schema 5 sessions as publication-observation replay.
They reject schemas 1-3 and untimed raw SSE input. No unresolved mapping investigation or recording-format change is required.
Implementation uses the accepted numeric BARO contract. Completion of #100 is not a prerequisite for this Task or its Subtasks.
Task completion still requires this Task's own playback, conversion, documentation, and validation criteria.

