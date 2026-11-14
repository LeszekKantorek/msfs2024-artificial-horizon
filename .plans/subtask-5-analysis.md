# Subtask 5 analysis — Existing documentation owners

## Authority and execution boundary


This is an implementation handoff for the matching Subtask in [the plan](browser-capture-evidence.md).
Source observations describe the inspected base `76174dfe6fb33eade3281034bf8ba54c3ea67f14`; selected changes below are implementation requirements, not claims of completed behavior.
Implement the specified behavior and run validation. Do not repeat discovery or choose a replacement design.
If the checkout differs materially or validation disproves an assumption, report the concrete conflict for plan correction. Do not silently invent a substitute.
No application changes or runtime validation were performed to prepare this analysis.


## Exact update inventory

| Owner | Inspected location | Required content |
| --- | --- | --- |
| [README](../README.md) | Scenario commands at38,64,94 | Rename input to test-scenario.jsonl; retain source/listen behavior |
| [CONTRIBUTING](../CONTRIBUTING.md) | Workflow command at121 | Same scenario rename |
| [Demo](../docs/sources/demo.md) | Baseline description15–45 | 138s pass,18s sweeps plus two60s phases; new name; held north window; native signals remain ordinary scenario data |
| [Scenario catalog](../docs/sources/demo-scenarios.md) | Baseline row7 and Flight Tour row8 | Short test scenario and verified UI coverage from Subtask4; retain Flight Tour's unit/manual evidence boundary |
| [SimConnect](../docs/sources/simconnect.md) | Logging command509 | Rename scenario in existing recorder example; keep existing logger ownership and flags |
| [Runtime testing](../docs/testing/runtime.md) | Startup example22 | Rename scenario |
| [Browser testing](../docs/testing/pfd.md) | Browser grouping75–81,commands100–105,coverage table108–110,module guidance below | Retained regression command; new capture command; trigger modules; separate app startup; all/manual termination; outputs and failures |
| [Validation index](../docs/testing/index.md) | Validation-by-change-scope matrix | Replace two-script workflow with retained regression plus applicable runtime trigger run |
| [Automated checks](../docs/testing/automated.md) | Browser tooling commands | Add tools/capture/*.test.mjs to tooling validation; retain complete build/release commands |
| [Evidence](../docs/testing/evidence.md) | Screenshot comparisons | Run identity, original PNG, observed sequence lookup, trigger reason/results, matching source/scenario/browser/viewport/revision; DOM brackets do not prove painted alert phase |
| [Lifecycle](../docs/efi/lifecycle.md) | Internal frame/render contract | Sequence retained through scheduler and status render; label flag; diagnostic footer geometry; clear on invalidation; timer redraw keeps sequence |
| [Selected altitude](../docs/efi/selected-altitude.md) | Coverage row140,manual acceptance163 | Replace198s claim with138s; approach/deviation flash and completion both directions; link scenario owner |
| [Vertical speed](../docs/efi/vertical-speed.md) | Acceptance139 |138s altitude/VSI pass;±1500 trend checks and±6000 faster transitions |

The existing project-validate-presentation skill still instructs agents to run both desktop scripts and pass a release executable to the runtime script. Replace that instruction with the retained regression suite and the separate recorder/capture workflow. Add the explicit authoring procedure and discovery routes specified below.
Do not rewrite historical issues or ADRs to remove historical names.

## Required agent-facing trigger authoring procedure

Subtask5 must publish durable instructions for future agents, not leave the knowledge only in local `.plans/` files.
Use the existing [presentation-validation skill](../.agents/skills/project-validate-presentation/SKILL.md) as procedure owner. Add a section titled `Author UI capture triggers` and link it from the skill's scenario-selection step.
Use the repository procedural-skill-creator procedure when editing this project-owned skill. Do not create another skill for the same validation workflow.

The section must give this executable sequence:

1. Apply the regression/trigger/scenario decision table from Subtask4 analysis, then select its instrument group and required dependencies. Preserve baseline inputs before the change.
2. Start from the complete trigger module example in `docs/testing/pfd.md`, or reuse the committed runtime definitions. State the intended event and expected appearance separately.
3. Select state/transition, once/rearm, dependencies, timeout, and finish mode using the documented decision rules. State that transition mode requires an observed false state before true.
4. Write a self-contained read-only observe function and pass constants via args. Use the selector/units map, guard missing nodes, and choose the case-specific availability guard. Live-reading cases require availability; loss/missing-source cases explicitly require unavailability. Pin target/alert kind where relevant. Read related values together.
5. Write a separate verify function returning named checks. Include every value needed by verification in the observation. Do not put all expected appearance into matches and thereby hide failed checks as missing events.
6. For boundaries use explicit before/after points. For flashing specify the phase and inspect the resulting PNG. Do not add scenario markers, application hooks, CDP logging, or tracing.
7. Save persistent test definitions under tests/browser/; keep one-off flight definitions in ignored .local/triggers/. Run the same screenshots.mjs CLI with --triggers for either source.
8. Validate syntax and the module contract, execute the chosen capture workflow, inspect failure evidence and PNGs, and report confirmed versus unconfirmed image states. Use exact sequence lookup in the separate raw SSE recording when available.

Keep the CLI/API schema and complete copyable example in the existing project documentation `docs/testing/pfd.md`; the skill links to that canonical section rather than duplicating the schema.
The example must contain export default, finish, the complete triggers array, real guarded DOM observations, and real named verification checks. Placeholder-only observe/verify functions do not satisfy this requirement.

### Agent discovery and acceptance

* Add a conditional row for creating/modifying UI capture triggers in [task guidance](../.context/task-guidance.md), pointing to the new skill section and canonical example.
* Add a visual-validation row in [PFD presentation map](../.context/pfd-presentation.md) pointing to the same authoring procedure. Keep public documentation free of links into .context/.
* Verify the route from task guidance to the procedure to the example and CLI command. A future agent must not need this conversation or local plan files.
* Verify the documented example's JavaScript syntax and contract against the implemented loader. Reuse the demonstrated target-up trigger run from Subtask4 as behavioral evidence; a prose-only edit does not require replaying the entire scenario.
* Acceptance requires the agent-facing procedure, both discovery links, the complete example, and removal of the skill's obsolete two-script/runtime-executable instruction.

## Maintaining selectors and publishing limitations

The authoring procedure must include this maintenance checklist whenever application DOM IDs, classes, aria-label formats, or data attributes used by observations change:

1. Search each changed literal with rg in `web/`, `tests/browser/`, `tools/capture/`, `docs/testing/pfd.md`, and the presentation-validation skill. Include local flight/rerun modules used for the acceptance run.
2. Update the actual UI producer, committed trigger consumers, retained regression selectors, and documented example together. Update the field map when it is retained as implementation evidence; do not create a production selector registry.
3. Preserve units, numeric precision, missing-node behavior, and the distinction between match conditions and verification. A renamed selector must not silently become zero or a permanently passing check.
4. Run retained affected regressions and capture-module tests. Run the relevant runtime trigger group with its predecessors, then the full acceptance set required by the validation matrix.
5. Check a deliberately missing required selector produces a recorded timeout/evaluation outcome with diagnostics, not a successful capture assertion. Recheck the complete published example against the implemented loader.

Publish the following limits explicitly in `docs/testing/pfd.md` beside the command and trigger examples:

* Trigger modules and imports are loaded at session startup. Editing a file during capture does not reload it; start a new capture session/output directory to use changed definitions.
* Capture does not perform automatic before/after pixel comparison or automatically confirm PNG content. The agent/operator inspects the images and records its conclusion separately.
* The capture tool opens its own persistent page at the supplied running application's URL. It does not attach to an existing tab, own/restart the application, or reconstruct UI/telemetry history before attachment.
* Raw SSE recording is separate. Existing tools/live/run.mjs owns the application it launches; it cannot attach its logger to an arbitrary already-running application. Do not start another application on an occupied port to imitate attachment.
* SIGINT/quit preserve completed evidence under the specified normal finalization path. This is not a power-loss, force-kill, or filesystem-failure durability guarantee.
* A trigger reports an observed UI condition. Availability loss and recovery can contribute to a false-to-true transition; it is not proof of a physical flight maneuver without supporting evidence.

Publish the EOF/quit/SIGINT matrix and result-preservation limits from Subtask2 without weakening its automated EOF rule.
Document these boundaries; do not implement hot reload, pixel diffing, an attach-mode logger, or another orchestration layer.

## Required working examples

Use the existing Playwright setup in docs/testing/pfd.md: PLAYWRIGHT_MODULE points to the local installation; BROWSER_CHANNEL may be msedge.
Show the existing SSE recorder (which starts the release application) and capture command in separate terminals, using the exact600s recording command from Subtask4 analysis. Do not start a second application on the same port.

```powershell
node tests/browser/pfd-browser.mjs
node tools/capture/screenshots.mjs --url "http://127.0.0.1:8080/?showSequence=true" --output-dir .local/screenshots/demo-01/ --triggers tests/browser/test-scenario.triggers.mjs
```

Document the plain two-flag invocation for manual captures during Demo or actual flight, with `shot`, empty line, and `quit`.
Use the exact trigger schema and semantics in [Subtask2 analysis](subtask-2-analysis.md). Include one working read-only DOM example, separate verify function, and failures that still produce PNGs.
Explain once/rearm, state/transition, dependency detection events, timeout activation, and `finish` behavior.
Document lifecycle setup from [Subtask4 analysis](subtask-4-analysis.md), including the separate fresh-client invocation.

## End-to-end Demo example to publish

Publish this as one continuous example in docs/testing/pfd.md, not as commands scattered across separate pages.
Prerequisites: complete release build, configured Playwright module/browser channel, implemented trigger definitions, and unused run names.
Run both terminals from the repository root. The existing recorder owns the application; do not start another application on the same port.

Terminal1 starts the application and records raw SSE:

```powershell
New-Item -ItemType Directory -Force .local/live | Out-Null
$appExe = Join-Path $PWD 'target/x86_64-pc-windows-msvc/release/msfs2024-artificial-horizon.exe'
node tools/live/run.mjs --exe "$appExe" --url http://127.0.0.1:8080/api/v1/events --output .local/live/demo-01.jsonl --duration-seconds 600 -- --source demo.dll --scenario demo-scenarios/test-scenario.jsonl --listen-address 127.0.0.1 --port 8080
```

Terminal2 starts capture against that application's URL:

```powershell
node tools/capture/screenshots.mjs --url "http://127.0.0.1:8080/?showSequence=true" --output-dir .local/screenshots/demo-01/ --triggers tests/browser/test-scenario.triggers.mjs
$captureExit = $LASTEXITCODE
Write-Output "Capture exit: $captureExit"
```

Wait for the final capture summary. It prints the absolute artifact paths, counts, failed IDs, and failed checks specified in Subtask2.
After capture completes, stop the recorder in terminal1 with Ctrl+C, or let its limit expire. Preserve its telemetry file; recorder cancellation is separate from the capture result.
Inspect the completed capture files in terminal2:

```powershell
$runDir = (Resolve-Path -LiteralPath '.local/screenshots/demo-01').Path
$session = Get-Content -LiteralPath (Join-Path $runDir 'session.json') -Raw | ConvertFrom-Json
$records = @(Get-Content -LiteralPath (Join-Path $runDir 'results.jsonl') | ForEach-Object { $_ | ConvertFrom-Json })
$session | Select-Object exitCode
$records | Where-Object { $_.type -eq 'trigger-terminal' } | Format-Table triggerId, status, reason
$captures = @($records | Where-Object { $_.type -eq 'capture' })
$captures | Format-Table captureId, triggerId, status, pngPath, stateEvidence, error
$captures | Where-Object { $_.verification.pass -eq $false } | ForEach-Object {
  $failedTrigger = $_.triggerId
  $_.verification.checks | Where-Object { -not $_.pass } | Select-Object @{Name='triggerId';Expression={$failedTrigger}}, name, actual, expected
}
```

A nonzero capture exit, failed named check, timeout, blocked, incomplete, cancelled, or session error is not a successful run.
A saved PNG can belong to a failed check. Conversely, exit0 and completed triggers establish execution/check success, not image confirmation.
If startup failed before artifact creation, use the terminal error; do not try to parse nonexistent files.

Open the desired PNG at `Join-Path $runDir <pngPath-from-record>`. Inspect the requested visual state and read its printed sequence.
Record confirmed/unconfirmed image evidence against that captureId. Do not use a DOM observation's sequence instead of reading the image.
After selecting the matching raw telemetry file, perform exact lookup:

```powershell
$sequenceText = Read-Host 'Sequence read from the inspected PNG'
[long]$pngSequence = 0
if (-not [long]::TryParse($sequenceText, [ref]$pngSequence) -or $pngSequence -lt 0) {
  throw 'Enter the nonnegative integer visible in the PNG'
}
$matchingRecords = @(Get-Content -LiteralPath '.local/live/demo-01.jsonl' | ForEach-Object { $_ | ConvertFrom-Json } | Where-Object { $_.sequence -eq $pngSequence })
if ($matchingRecords.Count -eq 0) {
  Write-Output 'UNMATCHED: the exact sequence is absent from this raw SSE recording'
} else {
  $matchingRecords | ConvertTo-Json -Depth 12
}
```

If the label is unreadable, the raw file is malformed, or the sequence is absent, report unresolved correlation. Never substitute the nearest sequence.
The acceptance report pairs the capture directory, exact raw file, launch arguments, application/DLL/scenario hashes, and trigger dependency manifest.
For before/after work repeat these steps with matched inputs and new run names, retaining both outputs.

## Copyable flight trigger example to publish

Save the following module as `.local/triggers/flight-altitude-band.mjs`.
The configurable values are in args: altitude band10000±50 feet and expected minimum climb500 feet per minute.
It first establishes readable SimConnect altitude, then detects a false-to-true transition of the available-altitude-band UI condition and checks climb separately. A wrong or missing VSI produces failed evidence instead of concealing the event. This is an observation of UI state, not an automatic claim of a physical altitude crossing.

```javascript
export default {
  finish: 'all',
  triggers: [{
    id: 'flight-ready',
    mode: 'state',
    repeat: 'once',
    after: [],
    timeoutMs: 300000,
    holdMs: 0,
    args: {},
    observe: () => {
      const instrument = document.querySelector('.instrument');
      const altitude = document.querySelector('#altitude');
      const match = /^Barometric altitude (-?\d+(?:\.\d+)?) feet$/.exec(altitude?.getAttribute('aria-label') ?? '');
      const feet = match ? Number(match[1]) : null;
      const ready = instrument?.dataset.source === 'simconnect' &&
        instrument?.dataset.available === 'true' && altitude?.dataset.state === 'available' &&
        Number.isFinite(feet);
      return { matches: ready, values: { ready, altitudeFt: feet } };
    },
    verify: values => ({
      pass: values.ready === true,
      checks: [{ name: 'readable SimConnect altitude', actual: values.ready,
        expected: true, pass: values.ready === true }],
    }),
  }, {
    id: 'flight-altitude-band',
    mode: 'transition',
    repeat: 'once',
    after: ['flight-ready'],
    timeoutMs: 300000,
    holdMs: 0,
    args: { altitudeFt: 10000, toleranceFt: 50, minimumClimbFpm: 500 },
    observe: args => {
      const instrument = document.querySelector('.instrument');
      const altitude = document.querySelector('#altitude');
      const vsi = document.querySelector('#vertical-speed');
      const vsiValue = document.querySelector('#vertical-speed-value');
      const altitudeMatch = /^Barometric altitude (-?\d+(?:\.\d+)?) feet$/.exec(altitude?.getAttribute('aria-label') ?? '');
      const vsiMatch = /^Vertical speed (-?\d+(?:\.\d+)?) feet per minute$/.exec(vsi?.getAttribute('aria-label') ?? '');
      const feet = altitudeMatch ? Number(altitudeMatch[1]) : null;
      const fpm = vsiMatch ? Number(vsiMatch[1]) : null;
      const values = {
        source: instrument?.dataset.source ?? null,
        available: instrument?.dataset.available === 'true',
        altitudeState: altitude?.dataset.state ?? null,
        altitudeFt: feet,
        minimumClimbFpm: args.minimumClimbFpm,
        vsiState: vsi?.dataset.state ?? null,
        verticalSpeedFpm: fpm,
        vsiText: vsiValue?.textContent ?? null,
        vsiVisibility: vsiValue ? getComputedStyle(vsiValue).visibility : null,
      };
      return {
        matches: values.source === 'simconnect' && values.available &&
          values.altitudeState === 'available' && feet !== null &&
          Math.abs(feet - args.altitudeFt) <= args.toleranceFt,
        values,
      };
    },
    verify: values => {
      const checks = [
        { name: 'VSI available', actual: values.vsiState, expected: 'available' },
        { name: 'minimum climb', actual: values.verticalSpeedFpm !== null &&
            values.verticalSpeedFpm >= values.minimumClimbFpm, expected: true },
        { name: 'positive climb text', actual: typeof values.vsiText === 'string' &&
            values.vsiText.startsWith('+'), expected: true },
        { name: 'VSI visible', actual: values.vsiVisibility, expected: 'visible' },
      ].map(check => ({ ...check, pass: check.actual === check.expected }));
      return { pass: checks.every(check => check.pass), checks };
    },
  }],
};
```

```powershell
node tools/capture/screenshots.mjs --url "http://127.0.0.1:8080/?showSequence=true" --output-dir .local/screenshots/flight-01/ --triggers .local/triggers/flight-altitude-band.mjs
```

Point the command at the already-running SimConnect application's URL. Capture opens its own persistent page; it does not restart the application or simulator, or take over an existing tab.
The readiness predecessor prevents initial page loading from supplying the band trigger's false baseline. The band trigger becomes active on the observation round after readiness detection and requires its own false then true. If its first active observation is already in-band and UI stays available, it waits for an observed exit/reentry. A subsequent availability loss followed by recovery in-band also satisfies the compound false-to-true condition; report that as UI recovery rather than a proved altitude crossing. Use mode state on the band trigger when matching the current state is intended.
For repeated observations, change only the band trigger's repeat to rearm and module finish to manual; keep flight-ready once. Each rearm requires the compound band condition to become false; this can be a band exit or loss of availability. End with quit. Do not use rearm with finishall.
No history before attachment is reconstructed. A fresh page also has fresh browser-local alert history.
For telemetry correlation, use the raw SSE recording from the recorder that already owns this application. If none exists, this capture can still provide UI evidence, but cannot add past SSE records.
Do not run tools/live/run.mjs on the same occupied port to attach logging: it starts a new application. A separately arranged future recorded run uses the existing recorder with the official SimConnect DLL source before capture attaches.
Use the same result inspection and PNG-confirmation steps as the Demo example; no default Demo target or scenario file is needed for this flight trigger.

## Publishing selection, rerun, and provenance instructions

Publish the regression/trigger/scenario decision table together with the [instrument selection table and dependency-preserving rerun example](subtask-4-analysis.md#selecting-checks-after-an-instrument-change) in docs/testing/pfd.md.
The agent-facing authoring procedure links to that section: retained regression remains the default UI suite, focused runtime subsets support iteration, and required acceptance runs retain full coverage.
Publish the [import provenance rule](subtask-4-analysis.md#provenance-for-trigger-imports) in docs/testing/evidence.md. Entry-module SHA256 alone is insufficient when the module imports a suite or helper.
The new examples, selector-maintenance checklist, explicit limitations, shutdown matrix, and final terminal summary are required Subtask5 acceptance evidence, not optional future improvements.

## Before/after evidence procedure

1. Save baseline inputs and build identity before the behavior change; use the same scenario content for the paired comparison.
2. Start tools/live/run.mjs separately with a new raw SSE filename; it owns the application. Start capture with a new output directory. Optional application --log records are separate diagnostics.
3. Run the same trigger definitions, viewport, browser, and label setting before and after the change.
4. Inspect PNGs for the requested visual state. Record changed/unconfirmed captures as such; repeat an explicit run if needed, retaining failed evidence.
5. Read sequence from each PNG visually or through available OCR. Look it up by exact top-level sequence in the paired raw SSE JSONL file. Separate subscribers can miss different publications: absence is unmatched, not a nearest-sequence substitution. Record run identity because sequence can restart.
6. Include before/after PNGs, trigger results, relevant log records, and validation outcome in future authorized PR evidence. This task does not authorize publishing a PR or comments.

Do not claim a sequence proves timer phase. Do not claim a new138s scenario is directly comparable to a screenshot from different198s input without identifying the matched case and inputs.

## Validation

Check local Markdown links/anchors and command paths. Search current documentation for obsolete scenario/runtime commands and198-second timing claims.
Verify examples against implemented CLI help and Subtasks1–4 evidence; do not rerun expensive application tests for prose alone.
Document complete case coverage across looping playback; do not promise captures from one pass after mid-pass attachment. Automatic records hold changed/unconfirmed states; the acceptance report holds visual confirmation.
Keep missing real-simulator evidence explicit. Do not describe unexecuted checks as passed.

## Paired-baseline limits for this tooling change

For Subtask1, capture the existing regression suite before the label change, then compare normal (flag disabled) captures after it. Enabled footer captures have no pre-feature equivalent; inspect their layout as new behavior rather than claiming an identical baseline viewport allocation.
For Subtask3, retain the original198s scenario locally as comparison input. Compare matched states from the preserved first18s and initial climb; compare new138s-only alert points to their specified requirements, not unrelated timestamps from the old scenario.
After this tooling change exists, future feature fixes can run the same new scenario and capture definitions unchanged before/after. This future workflow does not manufacture historical sequence-labelled images for the current change.
