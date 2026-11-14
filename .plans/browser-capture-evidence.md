# Add persistent UI-triggered screenshots

## Accepted scope

Keep the independent browser regression suite and replace the Demo-specific runtime browser script with `tools/capture/screenshots.mjs`.
Add `?showSequence=true` so screenshots can identify displayed telemetry in the existing logs.
Rename `demo-scenario.jsonl` to `test-scenario.jsonl` and shorten its playback while preserving the agreed runtime cases.
Keep `flight-tour.jsonl` as the longer flight presentation.
Update the current documentation after implementation.

| File or area | Role in this change |
| --- | --- |
| `tests/browser/pfd-browser.mjs` | Retained regression entry point. Extend focused coverage for the sequence label |
| `tests/browser/checks/` and `tests/browser/helpers/` | Retained controlled regression tests and fixtures |
| `tools/browser/session.mjs` and its tests | Retained shared browser utilities used by regression tests and the new capture tool |
| `tests/browser/pfd-runtime-browser.mjs` | Replaced by the new capture entry point and removed after its coverage is accounted for |
| `tools/capture/screenshots.mjs` | New persistent screenshot entry point for an already-running Demo or SimConnect application |
| `tools/live/` | Existing source of telemetry recordings for sequence lookup |
| `demo-scenarios/demo-scenario.jsonl` | Renamed and rebuilt as the shorter `test-scenario.jsonl` |
| `demo-scenarios/flight-tour.jsonl` | Retained longer flight presentation |

| Planning context | Value |
| --- | --- |
| Working directory | `.worktrees/browser-capture-evidence` |
| Branch | `change/browser-capture-evidence` |
| Inspected base | `76174dfe6fb33eade3281034bf8ba54c3ea67f14` |
| Parent | Local request, Persistent UI-triggered screenshots |
| Execution | One Terra Medium developer; execute Subtasks 1–5 in order |

This request authorizes a plan correction. Implementation and actual test results remain pending.

## Required implementation analyses

Read the matching analysis before executing each Subtask. These files contain the source findings, selected contracts, exact edits, and expected verification results.

| Subtask | Required analysis |
| --- | --- |
| 1 | [Rendered sequence](subtask-1-analysis.md) |
| 2 | [Persistent capture and trigger contract](subtask-2-analysis.md) |
| 3 | [138-second scenario and native checkpoints](subtask-3-analysis.md) |
| 4 | [Runtime assertion inventory and replacement triggers](subtask-4-analysis.md) |
| 5 | [Documentation owners and exact updates](subtask-5-analysis.md) |

The analyses are part of this plan. No discovery or consequential design choice is deferred to the implementing agent.
Source drift or contradictory validation evidence requires a specific plan correction, not an improvised replacement.
Verification of implemented behavior remains execution work; it is not a new discovery assignment.

## Evidence informing the design

* [Runtime checks](../tests/browser/pfd-runtime-browser.mjs) wait for UI conditions, assert values and geometry, then save PNGs.
* [Regression entry point](../tests/browser/pfd-browser.mjs) controls fixture input independently of native Demo playback.
* [Browser utilities](../tools/browser/session.mjs) already provide browser creation, screenshot saving, and cleanup.
* [Selected-altitude checks](../tests/browser/checks/selected-altitude-browser-checks.mjs) exercise alert history and 250 ms flashing phases.
* [Demo contract](../docs/sources/demo.md) describes independent tracks, repeated passes, and timing uncertainty.
* [Baseline scenario](../demo-scenarios/demo-scenario.jsonl) has a 198-second pass with shorter repeating tracks and long altitude phases.
* Its altitude track contains 18 seconds of digit-boundary sweeps followed by two 90-second selected-altitude approaches, one from each direction.
* [Selected-altitude timing](../docs/efi/selected-altitude.md#alert-transitions) requires five-second alerts. Shorter playback must preserve observable alert phases and completion.
* The current runtime script checks the beginning of playback before stopping and restarting the application. It does not validate the complete 198-second pass.
* [Scenario catalog](../docs/sources/demo-scenarios.md) identifies Flight Tour as unit-tested and manually accepted. Automatic UI coverage requires explicit new evidence.
* [Screenshot evidence](../docs/testing/evidence.md#screenshot-comparisons) requires matched conditions and visual inspection.

## Delivery order and shared constraints

Each Subtask belongs to the local request above. One developer executes the work in the listed order.

| Subtask | Deliverable | Prerequisites |
| --- | --- | --- |
| 1 | Optional rendered sequence label | None |
| 2 | Persistent screenshot command with all eight UI-trigger requirements | 1 for sequence-based capture validation |
| 3 | Short test scenario and updated native expectations | 2 for observable-state timing checks |
| 4 | Runtime trigger definitions, coverage migration, and retirement of the old script | 1–3 |
| 5 | Updated existing documentation | 1–4 and their evidence |

* Keep `pfd-browser.mjs`, its checks/helpers, and the shared browser utilities.
* Start the application separately. Do not add CDP logging, tracing, a process coordinator, application pause/resume hooks, or scenario checkpoint markers.
* Use existing `tools/live/` recordings for telemetry. Reading sequence from PNGs remains an analysis procedure; do not implement an OCR subsystem.
* Keep `pfd-runtime-browser.mjs` until Subtask 4 verifies replacement coverage. Subtask 3 updates its scenario references and timing-dependent expectations where required to keep the intermediate state usable.
* Execute the baseline prerequisites in Subtask1 analysis before browser edits and in Subtask3 analysis before scenario edits. Existing normal-mode images have a before/after comparison; the new footer and changed scenario segments have explicitly limited baseline equivalence. Subtask5 documents this later and must not defer those prerequisites. Retain exact local inputs, viewport, browser, and build identity.
* Follow [validation scope](../docs/testing/index.md#validation-by-change-scope), [application checks](../docs/testing/automated.md), and applicable independent review. Rebuild the complete application after changes to application code or build inputs.
* Each Subtask records actual commands, outcomes, inspected captures, and missing evidence. Validation belongs to the change that needs it.

## Subtask 1 — Display the rendered sequence on request

Required handoff: [Subtask 1 analysis](subtask-1-analysis.md).

### Outcome and prerequisites

Display the sequence of the rendered sample only when `?showSequence=true` is enabled.
There are no prerequisites. This Subtask changes application presentation, not capture tooling.

### Scope, context, and implementation guidance

| Source | Change |
| --- | --- |
| [web/app.js](../web/app.js) | Read `showSequence` during initialization and pass the option to the status view |
| [web/telemetry/client.js](../web/telemetry/client.js) | Carry accepted `sequence` into the internal frame |
| [web/status.js](../web/status.js) | Update a legible sequence label during `render(frame)` and clear it on invalidation |
| [web/panel.js](../web/panel.js) | Use its existing status-render call after instrument rendering |

Enable the label only for the exact parameter value `true`.
Update its value in the same synchronous DOM operation as the rendered indications.
Preserve sequence zero and clear the label when current readings are unavailable.
Timed redraws retain the contributing sequence.
Keep the text clear of required instrument indications and outside per-sample accessibility announcements.

Extend the retained regression tests with focused enabled/disabled label checks and screenshots.
Use the flag on the new diagnostic capture URL. Preserve other query parameters.

### Acceptance criteria

* [ ] The exact flag value `true` enables a legible label without obscuring required indications.
* [ ] The label uses the rendered frame's sequence, including zero, and clears when readings become unavailable.
* [ ] Timed redraws retain the contributing sequence; disabled mode preserves the normal presentation.
* [ ] Retained regression tests cover enabled/disabled behavior and invalidation.

### Validation

Rebuild the complete application and run the retained browser regression suite.
Inspect screenshots with the flag enabled and disabled, including unavailable data.
Verify label updates alongside instrument rendering and record the limits of DOM-to-image evidence.

## Subtask 2 — Implement persistent capture and UI triggers

Required handoff: [Subtask 2 analysis](subtask-2-analysis.md).

### Outcome and prerequisites

Provide one persistent page for manual screenshots and all eight accepted UI-trigger requirements.
Subtask 1 supplies the label used to validate screenshot-to-log lookup.
Keep Demo-specific trigger definitions and retirement of the old runtime script for Subtask 4.

### Scope and ownership

The new entry point takes the URL of an application started separately with Demo or SimConnect.
It owns its browser context, page, trigger state, and screenshot outputs.
Reuse `openBrowserSession` and `saveScreenshot` from the retained `tools/browser/session.mjs` module.
Keep that module importable by the retained regression suite.

Required invocation:

```powershell
node tools/capture/screenshots.mjs `
  --url "http://127.0.0.1:8080/?showSequence=true" `
  --output-dir .local/screenshots/demo-01/
```

### Persistent session

* Navigate once and retain the same page, telemetry subscription, preferences, and alert history across screenshots.
* Accept `shot` or an empty input line for a manual capture, and `quit` or Ctrl+C to close the session.
* Use consecutive filenames with trigger names where applicable, and print the saved absolute path.
* Serialize capture operations. Record trigger detection time and capture time when another capture delays a request.
* Keep trigger observation active during capture. Bound pending requests and report any overflow explicitly.
* Allow unavailable or disconnected UI to be captured for diagnosis.
* Close owned resources after fatal failure, input closure in manual mode, or manual browser closure.
* Automated trigger mode can finish when all required one-shot triggers have completed or timed out.
* Preserve existing files by requiring a new session output directory.
* Bound navigation, readiness, condition evaluation, and screenshot operations separately from the session lifetime.

Use the existing browser-channel setup. A visible session supports manual use.
Add the headless option specified in Subtask 2 analysis and preserve its current default for regression callers.

### UI trigger mechanism

Load an optional local trigger definition through `--triggers <path>`.
Use a local JavaScript module for read-only UI observations and predicates, accommodating existing SVG calculations.
The trigger definition supplies conditions and checks. Demo remains the source of scenario values.

The following required contract incorporates the accepted trigger requirements and mechanisms present in the current browser tests.
Use the exact module shape, CLI options, output records, and failure semantics in Subtask 2 analysis.

| Capability | Required behavior |
| --- | --- |
| Identity | Stable trigger name used in filenames and results |
| Observation | Read text, attributes, computed styles, numeric values, and SVG geometry in the page |
| Compound conditions | Evaluate related fields together in one synchronous browser observation |
| Comparisons | Equality, ranges, numeric tolerance, missing fields, and visible/unavailable states |
| Trigger mode | Match an existing state or fire on an observed false-to-true transition |
| Dependencies | Activate an ordered trigger after its prerequisite. Independent triggers observe concurrently |
| Repetition | Once per session or rearm after the condition becomes false |
| Readiness | Optionally require settled layout or a condition held for a defined duration |
| Deadline | Explicit timeout beginning when the trigger becomes active |
| Verification | Evaluate expected properties after detection and record failures while still attempting a diagnostic PNG |
| Capture result | Record observed values, trigger identity, verification outcome, and PNG path |

Observe actual UI updates, including timer-driven visibility changes and resize.
Use the nonoverlapping 25ms observation schedule specified in Subtask 2 analysis; validate coverage of 250ms alert phases.
Preserve observation history across screenshots. External navigation terminates the capture session with an explicit failure.
An entry condition already true at startup fires only under the configured state-match behavior.

Recheck the triggering state around capture. Preserve a changed-state capture with an explicit outcome.
For each capture, record observations before and after capture. Automatic records distinguish changed and unconfirmed image states; visual confirmation belongs to the acceptance report linked to the PNG.
Matching DOM observations around capture alone do not prove the painted state in the PNG, especially during flashing.
Inspect the captured image for the relevant visual condition before claiming that it depicts the triggering state; keep uncertainty explicit.
The sequence read from the PNG identifies captured telemetry, while trigger observations describe why capture was requested.
Record timeouts and evaluation failures and attempt a screenshot of the current page when possible.
A failed assertion must not prevent diagnostic capture.

The accepted requirements are mapped explicitly below.

| Requirement | Execution and evidence |
| --- | --- |
| 1. Persistent page and active triggers | Observe independent conditions concurrently; activate dependent points after their prerequisite event |
| 2. UI conditions | Support text, attributes, numbers, visibility, style, geometry, and read-only JavaScript predicates |
| 3. Activation semantics | Explicitly choose existing-state matching or an observed transition, including attachment during a flight |
| 4. Repetition | Explicitly choose once per session or rearming after leaving and reentering the condition, including looping Demo |
| 5. Timeout diagnostics | Record a missing point and capture the current state when possible |
| 6. Separate detection and verification | Detect the event, then assess properties such as color, text, and position; retain a PNG on verification failure |
| 7. Capture reason | Record trigger name, observed values, verification result, and PNG path; use the sequence visible in the image for telemetry lookup |
| 8. State at capture | Establish whether the PNG depicts the triggering state; retain changed or unconfirmed outcomes instead of reporting them as verified captures |

### Acceptance criteria

* [ ] The required command opens one persistent page against the supplied URL.
* [ ] The final terminal summary reports absolute artifact paths, capture counts, terminal trigger IDs, failed named checks, and exit code as specified in Subtask2 analysis.
* [ ] Manual and automatic captures preserve page identity and browser-local history.
* [ ] All eight requirements in the table above have verified behavior and explicit failure outcomes.
* [ ] Failed checks and timeouts retain diagnostic PNGs when capture is possible.
* [ ] Capture records distinguish changed and unconfirmed states; the acceptance report separately identifies visually confirmed PNGs.
* [ ] Shutdown, output protection, bounded capture queues, and failure cleanup work as specified. Automated EOF does not stop finishall; explicit quit and SIGINT follow the mode/event matrix.
* [ ] Handled interruption awaits queued record writes and final session output, retaining saved PNGs and parseable complete JSONL under normal filesystem operation.
* [ ] Shared browser utilities remain compatible with the retained regression suite.

### Validation

Use the explicit negative/boundary validation matrix in Subtask2 analysis, including CLI/module rejection, dependency cycles, navigation exceptions, mode-specific EOF, and SIGINT evidence preservation. Preserve its scheduler, failure, capture-race, and cleanup checks.
Run repeated manual and automatic captures against an already-running application.
Inspect captures of timed alert phases and record when the image does not confirm the triggering state.
Run affected shared-utility tests and regression callers if those utilities change.
This Subtask establishes the generic mechanism; full scenario coverage is verified in Subtask 4.

## Subtask 3 — Build the shorter test scenario

Required handoff: [Subtask 3 analysis](subtask-3-analysis.md).

### Outcome and prerequisites

Replace the 198-second baseline with the specified 138-second `test-scenario.jsonl`.
Subtask 2 supplies the capture mechanism for checking that important states remain observable.

### Scope, context, and implementation guidance

Rename `demo-scenarios/demo-scenario.jsonl` to `demo-scenarios/test-scenario.jsonl`.
Use it for short, deliberately selected runtime UI cases and screenshots.
Keep `flight-tour.jsonl` for the longer flight presentation.

1. Implement the supplied case-to-input and case-to-trigger mappings in Subtask 3 and Subtask 4 analyses.
2. Cover initial indications, turn and speed values, north crossing, altitude digit boundaries, level flight, climb/descent and trend, and selected-altitude alert transitions in both directions.
3. Use the exact segment durations and retained parallel tracks specified in Subtask 3 analysis.
4. Hold important states long enough for trigger detection and capture. Preserve the five-second alert duration and time to observe completion where required.
5. Implement the 138-second schedule, matching VSI values, and literal checkpoint expectations in Subtask 3 analysis.
6. Define UI triggers separately from scenario data. Keep ordinary Demo records without checkpoint markers or screenshot commands.
7. Update embedded/default scenario references, consumers, and independent native checkpoint expectations together. Do not retain a compatibility alias for the old filename.

The resulting pass lasts 138 seconds and covers the runtime cases enumerated in Subtask 4 analysis.
The independent `pfd-browser.mjs` suite retains detailed controlled-input regression coverage.
Process stop/restart remains external setup followed by UI observation; scenario records do not replace that setup.
The shorter pass enables complete runtime trigger coverage, rather than claiming a proportional speedup of the old runtime script.

Use [scenario parsing and native checkpoints](../crates/demo/src/scenario.rs), [DLL client tests](../crates/demo/tests/dll_client.rs), and the Demo contract when updating playback expectations.

Keep this Subtask's case-to-condition mapping as input to Subtask 4. Subtask 4 owns the committed runtime trigger definitions.
Update the old runtime script's scenario reference and affected expectations temporarily so the rename does not break it before migration.

### Acceptance criteria

* [ ] The renamed scenario completes its specified 138-second pass and maps every agreed runtime case to inputs and expected UI conditions.
* [ ] Altitude ramps and VSI agree; required alert phases and completion remain observable in both directions.
* [ ] Default/embedded references, consumers, and independent native expectations use the new name and values.
* [ ] Flight Tour retains its presentation role and existing checks.
* [ ] Scenario data contains no capture commands or checkpoint markers.

### Validation

Rebuild the complete application and Demo DLL.
Run affected parser, independent native checkpoint, and DLL client tests.
Observe the shortened playback with the persistent capture tool; inspect important states. The input schedule is138 seconds; record observed wall-clock times without inferring a source pass from them.
Run affected retained checks, including the old runtime entry point until Subtask 4 replaces it.
Full automatic case coverage is an acceptance requirement of Subtask 4.

## Subtask 4 — Migrate runtime coverage to UI-triggered captures

Required handoff: [Subtask 4 analysis](subtask-4-analysis.md).

### Outcome and prerequisites

Exercise the short scenario through committed UI trigger definitions and replace `pfd-runtime-browser.mjs` without losing its required coverage.
Subtasks 1–3 supply the label, capture mechanism, and test data.
This is the integration and coverage migration deliverable, including removal of the old runtime entry point.

### Scope, context, and implementation guidance

Use the completed assertion inventory in Subtask 4 analysis; it identifies each old assertion group and its destination.
Move its screenshot conditions into Demo-specific trigger definitions used by the generic capture tool.
Retain the distinction between UI observations and separately recorded telemetry evidence.

| Current runtime behavior | Replacement responsibility |
| --- | --- |
| Right turn, bank, speed values, trend disappearance | Compound UI trigger and verification |
| North crossing and heading-bug alignment | UI trigger with numeric SVG verification |
| Selected altitude becoming available | State trigger with target, alert, and bug checks |
| Altitude hold, climb, and trend | Ordered UI conditions with text, ranges, and paint checks |
| Source loss and fresh recovery | Transition triggers on the same page |
| Connected HTTP with unavailable SimConnect | Combined source, transport, availability, and visible-message checks |
| JavaScript page errors and actual browser module loading | Capture-run validation records |
| GET/HEAD, MIME, health, and obsolete-route checks | Retain the identified HTTP tests and add the missing true-airspeed HEAD case specified in Subtask 4 analysis |
| Process stop/restart and missing-DLL setup | Explicit test setup using existing application/process facilities, followed by UI trigger observation |
| Separate direct SSE reads | Existing telemetry evidence and applicable contract tests |

Implement each replacement in the supplied assertion inventory before deleting `pfd-runtime-browser.mjs`.
Record execution evidence against those rows; failed or missing evidence prevents completion.
The retained regression suite continues to exercise controlled inputs, paint order, node identity, layout, and controlled-clock behavior.

Implement the additional full-pass trigger definitions enumerated in Subtask 4 analysis, using Subtask 3's specified input schedule.
Keep trigger definitions separate from scenario data and reusable through the generic capture entry point.
Update executable validation callers when removing the old entry point. Subtask 5 updates the owning prose and procedures.

### Acceptance criteria

* [ ] Every agreed short-scenario case produces a trigger outcome and required screenshot evidence during looping playback. Evidence may span passes after mid-pass attachment; do not claim a single-pass capture.
* [ ] Every old runtime assertion has its specified replacement verified before script removal.
* [ ] Source loss, recovery, and unavailable SimConnect are observed on the same page using explicit external setup. Their unavailable-state guards are separate from live-reading guards.
* [ ] Trigger reads follow the concrete selector/attribute/unit map, including digit columns, alert phases, and SVG geometry.
* [ ] Timeout, failed verification, and changed-state captures leave useful diagnostic evidence.
* [ ] The old runtime entry point is removed; retained regression tests and shared utilities still pass.
* [ ] Evidence identifies the application run and uses the sequence visible in PNGs for lookup in existing logs.
* [ ] Focused instrument checks and one-case reruns retain transitive trigger dependencies; required final coverage remains explicit.
* [ ] The acceptance report identifies entry modules and their transitive local helper/data imports by path and SHA256.

### Validation

Run the complete applicable suite after integration, including retained browser regressions and the new complete case-set capture run across looping playback.
Inspect the PNGs and verify required trigger outcomes against the case mapping.
Exercise source loss/recovery and record external setup commands and evidence.
Check screenshot-to-log lookup with the existing logger and identify any unmatched or unreadable sequence.
Use actual simulator flight evidence when available and state its limits separately; Demo evidence does not establish real-flight acceptance.
Obtain applicable independent review and record unresolved coverage before claiming completion.

## Subtask 5 — Update the existing documentation

Required handoff: [Subtask 5 analysis](subtask-5-analysis.md).

### Outcome and prerequisite

Document the retained regression suite and the new persistent runtime capture workflow accurately. Publish the agent-facing trigger-authoring procedure and its discovery routes specified in Subtask5 analysis.
Subtasks 1–4 supply implemented behavior and validation evidence. The parent is the same local request.

### Scope and context

| Existing owner | Update |
| --- | --- |
| [Browser testing](../docs/testing/pfd.md) | Retained regression command, new runtime capture command, trigger definition, manual capture, and output paths |
| [Validation scope](../docs/testing/index.md#validation-by-change-scope) | Replace the two-script instruction with the regression suite and applicable runtime trigger checks |
| [Evidence](../docs/testing/evidence.md#screenshot-comparisons) | Trigger outcomes, original PNGs, sequence lookup, and matched before/after conditions |
| [Frame lifecycle](../docs/efi/lifecycle.md) | Internal sequence field and visible label behavior |
| [Demo contract](../docs/sources/demo.md) and [scenario catalog](../docs/sources/demo-scenarios.md) | New test-scenario name, measured duration, covered cases, and distinction from Flight Tour |
| [Selected altitude](../docs/efi/selected-altitude.md) | Revise acceptance instructions that reference the 198-second baseline |

Read [existing logging instructions](../docs/sources/simconnect.md#bounded-development-observation) and reference their current commands.
Apply the complete reference inventory in Subtask 5 analysis. Verify that those edits leave no obsolete current instructions.

Agent procedure ownership: extend `.agents/skills/project-validate-presentation/SKILL.md` with `Author UI capture triggers`; route agents from `.context/task-guidance.md` and `.context/pfd-presentation.md`.
Keep the canonical module schema and complete working example in `docs/testing/pfd.md`. Local planning analyses are not the durable user/agent instructions.

### Implementation guidance

1. Describe `pfd-browser.mjs` and its helpers as independent UI regression tests.
2. Document launching the application with Demo or SimConnect and opening its URL once with the new capture command.
3. Provide a working UI-trigger example based on a migrated runtime checkpoint.
4. Explain existing-state versus transition triggers, repetition, dependencies, timeouts, and capture-race outcomes.
5. Describe reading sequence from the PNG visually or using available OCR, then locating it in the existing run's log.
6. Extend before/after guidance with matching source, scenario, viewport, browser, and build identity.
7. Update commands and links that previously invoked `pfd-runtime-browser.mjs`.
8. Replace current references to `demo-scenario.jsonl` with the new test scenario and update affected default-launch examples and timing descriptions.
9. Document the case-to-trigger mapping and the limits of capture-state evidence, including changed and unconfirmed PNGs.

The acceptance report maps each capture directory to the existing recorder's raw SSE file and application run. Application --log output is optional diagnostics, not a replacement for raw SSE records. Sequence values can recur after restart.
Unreadable labels and missing records remain explicit unresolved correlations.
OCR remains an analysis procedure documented here.

### Acceptance criteria

* [ ] Documentation clearly assigns regression testing to the retained suite and runtime capture to the new command.
* [ ] Examples match the implemented CLI and trigger interface.
* [ ] One end-to-end Demo example covers both terminals, result inspection, PNG confirmation, and exact raw-SSE lookup.
* [ ] A complete flight trigger example uses configurable ranges and separate verification, with attachment/logger limits stated.
* [ ] Published selection and rerun instructions identify the instrument groups and preserve prerequisite events without adding CLI filter options.
* [ ] Agent instructions state when to extend regression checks, add a trigger, or change scenario inputs, and what to update after selectors change.
* [ ] Documentation explicitly states no trigger hot reload, no automatic pixel comparison, separate logger ownership, and ordinary-cancellation preservation limits.
* [ ] The flight example establishes readiness before its band transition and distinguishes an observed UI transition from a physical altitude crossing.
* [ ] The presentation-validation skill explains how agents create and verify triggers, with working discovery links from the existing context maps.
* [ ] A complete executable module example replaces placeholder-only callbacks; agents can follow the published procedure without this chat or local plan files.
* [ ] The skill no longer prescribes the removed two-script/runtime-executable workflow.
* [ ] The same capture workflow is documented for Demo and SimConnect.
* [ ] Removed runtime-script invocations are replaced in current procedural sources.
* [ ] Screenshot/log lookup and before/after requirements describe actual tooling and evidence limits.
* [ ] Scenario documentation distinguishes the short test pass from Flight Tour and matches the measured duration and verified cases.

### Validation

Verify local links, anchors, command arguments, filenames, and incoming references.
Walk through the new documentation using Subtasks 1–4 evidence.
Apply documentation-only checks when changes affect prose alone, and record any unavailable evidence.

## Review corrections

The independent review and author checks identified and corrected the following planning defects:

* Use the existing raw SSE recorder for sequence correlation; schema4 application diagnostics do not record every publication.
* Specify paired before/after digit-boundary captures, with exact numeric bands and dependencies.
* Pin every alert phase to available UI and the exact target/kind.
* Describe complete case coverage across looping playback without promising all images came from one pass.
* Keep automatic capture-state evidence separate from visual confirmation in the acceptance report.
* Specify trigger validation, verification exceptions, queue failures, timeouts, observation locking, termination, and result records.
* Put baseline collection before the corresponding edits in Subtasks1 and3, rather than deferring those prerequisites to documentation.

The earlier independent static review completed with follow-up corrections. A later question-based review identified further gaps.
The corrections for questions1–5 and11 remain in the terminal-summary, instrument-selection, rerun, end-to-end example, flight example, and import-provenance sections. The subsequent corrections for questions12–22 are specified in the UI-field map, case-specific availability guards, negative validation matrix, termination/output-finalization contract, change-selection rules, selector-maintenance procedure, and explicit limitations. These are corrected specifications; their implementation tests have not run.
The existing alert state machine was also checked with the proposed ideal50ms schedule, as recorded in Subtask3 analysis.
These checks do not establish browser/native runtime behavior; implementation validation remains mandatory.

## Status

The plan separates application presentation, generic capture, scenario data, runtime coverage migration, and documentation into five deliverables.
All eight trigger requirements and the accepted regression/runtime split remain in scope.
Implementation, script removal, and execution validation remain pending.
