# Subtask 3 analysis — 138-second test scenario

## Authority and execution boundary

This is an implementation handoff for the matching Subtask in [the plan](browser-capture-evidence.md).
Source observations describe the inspected base `76174dfe6fb33eade3281034bf8ba54c3ea67f14`; selected changes below are implementation requirements, not claims of completed behavior.
Implement the specified behavior and run validation. Do not repeat discovery or choose a replacement design.
If the checkout differs materially or validation disproves an assumption, report the concrete conflict for plan correction. Do not silently invent a substitute.
No application changes or runtime validation were performed to prepare this analysis.

## Baseline prerequisite before scenario edits

Before renaming or changing the scenario, preserve the original198s file in ignored `.local/` storage and record its SHA256.
With the Subtask1 label and Subtask2 capture tool, retain baseline images for the unchanged initial18s boundaries, north crossing, target initialization, and initial1500fpm climb, together with the separate raw SSE recording.
Use manual capture for this prerequisite; Subtask4 trigger definitions are not required yet. A missed north crossing is missing baseline evidence, not permission to add a marker to the original scenario.
After the change compare matched instrument states and inputs. New held-north and shortened alert segments are validated against their specified expectations; do not compare unrelated elapsed timestamps.
Subtask5 documents this procedure but does not own its execution timing.

## Existing evidence and selected duration

[The baseline](../demo-scenarios/demo-scenario.jsonl) uses 50ms frames, six altitude sweeps totaling 18s, and two 90s selected-altitude phases.
[Selected-altitude logic](../web/altitude/selected.js) flashes for 5000ms in 250ms phases. The first valid target establishes bands silently.
Keep the initial 18s altitude/VSI sweeps exactly. Replace the two 90s phases with the following two 60s phases: total **138 seconds**, not an unspecified shorter duration.
This timing is a specified input schedule. Wall-clock capture performance remains validation evidence, not an assumed guarantee.

## Altitude/VSI/target schedule

Times are seconds from the start of each pass. Intervals are half-open.

| Time | Altitude feet | VSI fpm | Target feet | Purpose |
| --- | --- | --- | --- | --- |
| 0–18 | Existing six sweeps unchanged | Existing gap/hold pairs unchanged | Gap | Signed digits and 1000/10000 boundaries |
| 18–24 | Hold 8500 | Gap for first 50ms, then 0 | 10000 | Silent target initialization and level flight |
| 24–48 | Ramp 8500→9100 | +1500 | 10000 | Existing climb/trend geometry; 1000ft entry at t=44 |
| 48–54 | Hold 9100 | 0 | 10000 | Complete 1000ft flash by approximately t=49 |
| 54–62 | Ramp 9100→9900 | +6000 | 10000 | 200ft entry at t=61 |
| 62–68 | Hold 9900 | 0 | 10000 | Complete 200ft flash by approximately t=66 |
| 68–72 | Ramp 9900→10300 | +6000 | 10000 | Departure beyond 10200 just after t=71 |
| 72–78 | Hold 10300 | 0 | 10000 | Complete deviation flash; steady yellow before t=78 |
| 78–84 | Hold 2500 | Gap for first 50ms, then 0 | 1000 | Target change resets history; silent initialization |
| 84–108 | Ramp 2500→1900 | -1500 | 1000 | 1000ft entry at t=104 |
| 108–114 | Hold 1900 | 0 | 1000 | Complete first approach flash |
| 114–122 | Ramp 1900→1100 | -6000 | 1000 | 200ft entry at t=121 |
| 122–128 | Hold 1100 | 0 | 1000 | Complete second approach flash |
| 128–132 | Ramp 1100→700 | -6000 | 1000 | Departure below 800 just after t=131 |
| 132–138 | Hold 700 | 0 | 1000 | Complete deviation flash; steady yellow before wrap |

The jump at 78s and loop boundary are deliberate test transitions, not a flight dynamics model. Use the existing 50ms VSI gap convention at the 78s jump.
Set header `duration_us=138000000`; target track is gap18000ms, hold10000 for60000ms, hold1000 for60000ms.
Set nonrepeating constant AP/radio/VS0/VS1/VR tracks to138000ms with current values unchanged.

## Other tracks and north capture

Keep pitch, bank, ball, turn, barometric setting, IAS, TAS, GS, and heading-target tracks unchanged, including their 18s repetition.
For heading retain 0–10s unchanged; replace 10–14s with ramp354→360 for1500ms, hold360 for1000ms, ramp0→-6 for1500ms. Retain final354 hold4000ms.
This provides a one-second normalized north window while selected heading is zero. It makes the screenshot condition observable beyond the old single-frame crossing.
The turn-rate track remains an independent test input; the scenario does not claim coordinated flight dynamics.

## Exact consumer edits

Rename the file with no alias. Update filename references in:

* `src/application.rs` default-path test.
* `crates/demo/src/scenario.rs` parser tests and independent checkpoints.
* `crates/demo/tests/dll_client.rs` both baseline consumers.
* `tests/cli.rs` and `tests/simconnect_cli.rs` scenario paths.
* `tools/live/run.test.mjs` rebuilt-Demo integration input.
* `tests/browser/pfd-runtime-browser.mjs` temporary default path until Subtask 4 deletes it. Its first25s checks still use the same altitude and VSI values; no timing rewrite is required for those checks.

Subtask 5 owns the prose references listed in its analysis.

## Independent checkpoint expectations

In `default_matches_independent_native_checkpoints`, retain existing5s IAS75,8s TAS160,10s GS170,2.95s altitude120,3s altitude980,18s altitude8500/target10000,7s heading351,6s baro30.12,10s ball127,17s target absent,0s VSI absent, and2s pitch assertions.
Replace target1000 checkpoint108s with78s. Replace wrap198s with138s.
Add independently specified checkpoints:

| Time s | Altitude | VSI | Target |
| --- | --- | --- | --- |
| 24 | 8500 | 1500 | 10000 |
| 44 | 9000 | 1500 | 10000 |
| 48 | 9100 | 0 | 10000 |
| 61 | 9800 | 6000 | 10000 |
| 68 | 9900 | 6000 | 10000 |
| 72 | 10300 | 0 | 10000 |
| 78 | 2500 | absent | 1000 |
| 84 | 2500 | -1500 | 1000 |
| 104 | 2000 | -1500 | 1000 |
| 121 | 1200 | -6000 | 1000 |
| 132 | 700 | 0 | 1000 |

Native heading at11.5s and12s is360; at13.25s it is-3. Keep expected values literal and independent of the scenario evaluator.
At138s the pass wraps; expected IAS0 and target absent.

## Verification

Rebuild the complete workspace and Demo DLL. Run workspace tests, retained browser regression, the temporarily retained runtime check, and tools/live tests with `LIVE_TEST_EXE` set.
Use Subtask 2 captures to inspect held states. Subtask 4 owns committed automatic trigger definitions and complete case coverage.
Flight Tour and its independent checkpoint file are unchanged.

## Planning verification performed

A Node check ran the existing `advanceAlert` and `alertPresentation` functions against the proposed schedule with ideal50ms samples.
It produced these transition times in seconds: up1000 flash44→49; up200 flash61→66; updeviation71.05→76.05; target reset78; down1000 flash104→109; down200 flash121→126; downdeviation131.05→136.05.
The check confirmed the scheduled alerts finish before the next target change or wrap. It does not validate native delivery, SSE coalescing, browser paints, or screenshot timing; those remain specified execution checks.
