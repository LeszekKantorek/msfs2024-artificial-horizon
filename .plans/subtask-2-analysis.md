# Subtask 2 analysis — Persistent capture contract

## Authority and execution boundary

This is an implementation handoff for the matching Subtask in [the plan](browser-capture-evidence.md).
Source observations describe the inspected base `76174dfe6fb33eade3281034bf8ba54c3ea67f14`; selected changes below are implementation requirements, not claims of completed behavior.
Implement the specified behavior and run validation. Do not repeat discovery or choose a replacement design.
If the checkout differs materially or validation disproves an assumption, report the concrete conflict for plan correction. Do not silently invent a substitute.
No application changes or runtime validation were performed to prepare this analysis.

## Existing reusable code

[session.mjs](../tools/browser/session.mjs) resolves `PLAYWRIGHT_MODULE`, launches Chromium with `BROWSER_CHANNEL`, creates one context/page, saves PNGs with a 5000ms timeout, and provides idempotent close with a 1000ms bound.
Add `headless=true` to `openBrowserSession` options; existing callers retain current behavior. The new CLI passes `headless=false` unless `--headless` is supplied.
Keep process management outside this tool. Its only entry point is `tools/capture/screenshots.mjs`; trigger modules are imported definitions, not additional executables.

## CLI and outputs selected for implementation

| Interface | Contract |
| --- | --- |
| Required flags | `--url <http-or-https-url>` and `--output-dir <path>`; reject credentials, missing values, duplicates, and unknown flags |
| Optional flags | `--triggers <module-path>`, `--headless`; default viewport 568x240, device scale 1 |
| Manual input | `shot` or empty line captures; `quit` requests orderly shutdown in either mode. EOF requests orderly shutdown only in manual mode; Ctrl+C follows the cancellation contract below |
| Automated mode | Trigger module sets `finish: 'all'` or `'manual'`. Ignore stdin EOF for `all`; exit after all one-shot triggers are terminal and queue drained; reject an empty triggers array |
| Paths | Require a nonexistent session directory, create its parent, and reserve the session directory without recursive overwrite. PNG names `0001-<trigger-id>.png` or `0001-manual.png` |
| Records | Append `results.jsonl`; write `session.json` containing URL, viewport, browser version, start/end times, trigger module path/hash, and exit result. Output directory identifies the capture run |
| Limits | Navigation 10000ms; observation 1000ms; screenshot 5000ms; queue 32 requests; poll every 25ms after previous observation completes; no overlapping observation calls |
| Exit | 0: requested captures/checks completed; 1: invalid input, failed check, timeout, overflow, browser error, or capture failure; 130: Ctrl+C when cancellation is otherwise successful; any run/output/cleanup failure requires 1. Orderly quit/manual EOF before required once triggers complete records incomplete and returns 1; rearming triggers are not required to finish on quit |

## Trigger definition shape

A local ES module exports a default object with `finish`, `triggers`, and optional `expectedModules: string[]`. At run completion compare expectedModules to the exact loaded same-origin JavaScript pathname set; missing/extra paths fail the run. Resolve paths from response URLs (ignore query/fragment).
Each trigger has this shape:

```javascript
{
  id: 'right-turn',
  mode: 'state', // 'state' or 'transition'
  repeat: 'once', // 'once' or 'rearm'
  after: [], // IDs whose detection events must have occurred
  timeoutMs: 300000,
  holdMs: 0,
  args: {}, // JSON-serializable values passed to observe
  observe: args => ({ matches: false, values: {} }),
  verify: values => ({ pass: true, checks: [] })
}
```

* `observe` is a self-contained synchronous browser function. It reads DOM, computed style, visibility, numeric values, and geometry. It cannot depend on imported closures. Send it and `args` through Playwright evaluation.
* `values` is JSON-serializable and contains all values required by `verify`. Read related fields in one observation. `verify` runs synchronously in Node against that snapshot and returns named `{name, pass, actual, expected}` checks.
* Conditions and verification are separate: an alert can match while its color check fails. Always attempt its PNG after a failed verification.
* Reject duplicate/unsafe IDs (allow lowercase letters, digits, hyphens), unknown dependencies, dependency cycles, noninteger or nonpositive timeoutMs, noninteger or negative holdMs, holdMs >= timeoutMs, and nonfunctions before navigation.
* Modules are trusted local diagnostic code. The read-only rule is a usage contract, not a JavaScript sandbox.
* `after` means detection, not screenshot completion. Arm dependent triggers immediately when all prerequisite detection events have occurred. A prerequisite that terminates without detection makes its descendants `blocked`.
* `state` can match its first observation. `transition` requires observing false after activation before observing true. `holdMs` requires continuously observed truth; any false resets it.
* `once` disables after its first detection. `rearm` requires a subsequent false before another detection. No deadline runs while waiting to observe false after a detection. Its timeout restarts on that false observation. A timeout or predicate failure terminates the trigger rather than automatically retrying it. Reject `finish:'all'` with rearming triggers.
* Each active trigger's timeout begins at activation. On timeout, record values and attempt a diagnostic PNG. Evaluation exceptions terminate that trigger and also attempt a PNG.
* Manual requests and trigger detections share one FIFO queue. Observation continues during screenshots. A full queue records overflow with trigger ID and observed values; mark that capture failed without replacing earlier requests. Continue observing other triggers; exit1 when the run finishes. Never silently drop a request.
* After initial goto finishes, a new main-frame document navigation is a fatal session event, with a diagnostic record. Ignore subframe navigation and same-document fragment/history changes; permit redirects belonging to initial goto. This tool neither navigates for each screenshot nor adds a reload command. Fresh-client checks use a separate explicit capture invocation.

## Complete reference module

This is the reference file shape for a single real UI condition. Save it as a trigger module when executing the handoff; the capture CLI loads its default export.
It implements the `target-up` case from Subtask4. It reads the selected and current altitude from UI, then separately verifies the initial target presentation.
The example does not replace Subtask4's complete case inventory.

```javascript
export default {
  finish: 'all',
  triggers: [{
    id: 'target-up',
    mode: 'state',
    repeat: 'once',
    after: [],
    timeoutMs: 300000,
    holdMs: 0,
    args: { target: 10000, altitude: 8500 },
    observe: args => {
      const instrument = document.querySelector('.instrument');
      const selected = document.querySelector('#selected-altitude');
      const value = document.querySelector('#selected-altitude-value');
      const altitude = document.querySelector('#altitude');
      const bug = document.querySelector('#selected-altitude-bug');
      const required = { instrument, selected, value, altitude, bug };
      const missing = Object.entries(required)
        .filter(([, node]) => node === null).map(([name]) => name);
      if (missing.length) return { matches: false, values: { missing } };

      const altitudeLabel = altitude.getAttribute('aria-label') ?? '';
      const parsed = /^Barometric altitude (-?\d+(?:\.\d+)?) feet$/.exec(altitudeLabel);
      const feet = parsed ? Number(parsed[1]) : null;
      const transform = bug.transform.baseVal.consolidate();
      const values = {
        expectedTarget: args.target,
        available: instrument.dataset.available === 'true',
        selectedState: selected.dataset.state,
        altitudeState: altitude.dataset.state,
        altitude: feet,
        targetText: value.textContent,
        targetLabel: selected.getAttribute('aria-label'),
        alert: selected.dataset.alert,
        flashing: selected.dataset.flashing,
        valueVisibility: getComputedStyle(value).visibility,
        bugDisplay: getComputedStyle(bug).display,
        bugVisibility: getComputedStyle(bug).visibility,
        bugY: transform ? transform.matrix.f : null,
      };
      return {
        matches: values.available && values.selectedState === 'available' &&
          values.altitudeState === 'available' && feet === args.altitude &&
          values.targetText === String(args.target),
        values,
      };
    },
    verify: values => {
      const expected = String(values.expectedTarget);
      const checks = [
        { name: 'target text', actual: values.targetText, expected },
        { name: 'target label', actual: values.targetLabel,
          expected: `Selected altitude ${expected} feet` },
        { name: 'silent initialization', actual: values.alert, expected: 'none' },
        { name: 'not flashing', actual: values.flashing, expected: 'false' },
        { name: 'visible value', actual: values.valueVisibility, expected: 'visible' },
        { name: 'visible bug', actual: values.bugVisibility, expected: 'visible' },
        { name: 'bug enabled', actual: values.bugDisplay !== 'none', expected: true },
        { name: 'bug upper limit', actual: values.bugY, expected: 14.5 },
      ].map(check => ({ ...check, pass: check.actual === check.expected }));
      return { pass: checks.every(check => check.pass), checks };
    },
  }],
};
```

### Applying the reference to the complete definitions

1. Create the three definition modules named in Subtask4 analysis. Each has one default export with finishall and a triggers array.
2. Use the exact IDs, modes, dependencies, bounds, and checks in its inventory and guard/boundary sections. Expand paired and hidden/visible cases as specified there.
3. Keep DOM reads inside self-contained observe functions. Pass constants through args; do not close over a Node helper inside a browser function.
4. Put all values needed for checks in the JSON observation. Keep expected appearance checks in verify so a wrong color, text, or geometry produces a failed check and diagnostic PNG.
5. Reuse Node-side verification helpers where useful; they cannot read the page. Reuse an observe function only when all varying values are explicit args.
6. Apply the module validation and tooling checks specified above. Then execute the scenario/lifecycle acceptance procedure from Subtask4.

These are coding steps for specified behavior. The implementing agent does not choose a new trigger language, infer missing scenario phases, or design another event mechanism.

## Capture-state evidence

For each detection, retain detection time, values, verification, before/after capture observations, filename, screenshot start/end times, and errors.
Reevaluate the matching condition immediately before and after screenshot capture.
Record `stateEvidence:'changed'` if a surrounding observation no longer matches; otherwise record `stateEvidence:'unconfirmed'`. Missing/failed brackets are unconfirmed with an error; a failed screenshot has no image evidence. Do not compare only a broad matches Boolean when a phase was requested: phase belongs in that trigger predicate.
Never automatically mark PNG content as verified from DOM brackets. A human/agent inspecting the PNG records visual confirmation in the acceptance report.
Do not freeze the app, disable animations, retry silently until success, or equate sequence equality with alert-phase equality.
Capture records describe why a capture happened. Sequence for telemetry lookup is read from the image during evidence analysis.

Listen for `pageerror` and failed/non-JavaScript `.js` responses; preserve these as run failures. Keep loaded module paths/status/MIME in results for Subtask 4's explicit module-set check.
Browser closure, output failure, or unresponsive evaluation terminates the run with bounded cleanup. Preserve partial files and primary error information.

## Focused tests

Add `tools/capture/screenshots.test.mjs`; keep helpers internal or exported from the same entry point with an import-safe main guard.
Use deterministic observations and a fake clock for state/transition, hold reset, rearm, dependency activation, blocked dependencies, and deadlines.
Use stubbed page/screenshot operations for FIFO, overflow, verification failure with PNG, browser closure, output failure, and cleanup.
Use a real browser on a small test page for computed geometry, 250ms visibility phases, repeated screenshots without navigation, and capture-state mismatch. This fixture belongs to tooling tests, not the Demo scenario or application.
Run `node --test tools/browser/*.test.mjs tools/capture/*.test.mjs` and retained regressions when shared helpers change.

## Evaluation, termination, and record details

* All fields shown in the trigger shape are required. `finish` is required when a module is supplied. Without a module use manual mode with zero triggers. Reject unsupported mode/repeat/finish values before browser creation.
* An observation round evaluates all currently active observe functions in one browser task, each with its args and individual try/catch. Node processes returned observations in module-array order. Newly activated dependents join the next round, avoiding order-dependent same-round detection.
* Before/after screenshot observations use the same browser-evaluation lock as polling. Await only the short observation under that lock, not screenshot completion. Polling rounds continue while a screenshot is in flight.
* Validate returned `{matches:boolean, values:JSON}`. Null/invalid results, Promise results, or thrown predicates are evaluation failures. Missing DOM during readiness is represented explicitly by matchesfalse and values listing missing selectors; supplied predicates must guard absent nodes.
* Validate verification `{pass:boolean, checks:[{name:string,pass:boolean,actual:JSON,expected:JSON}]}`. Overall success requires pass and every check.pass. A thrown/invalid verifier is a verification failure and still requests diagnostic PNG. Call it once for the detection observation, not repeatedly for screenshots.
* Callbacks are trusted synchronous terminating functions. Enforce the1000ms browser-round watchdog with a host timer; on expiry close the owned page/context, record fatal evaluation timeout, and stop. Do not enqueue further evaluations behind an unresponsive browser. Diagnostic PNG is unavailable in that fatal case.
* Deadline processing occurs before newly returned observations at the same monotonic timestamp. Blocked triggers are terminal failures with dependency IDs, no invented observation or redundant PNG. A missed deadline attempts one diagnostic capture using the last observation plus fresh capture brackets.
* A trigger is terminal for finish purposes only after its once capture succeeds/fails, or its failure diagnostic attempt finishes. Parent detection is still sufficient to activate dependencies. Record detection immediately so external setup can see progress.
* Record types in results.jsonl are `detection`, `capture`, `trigger-terminal`, `page-error`, `module-response`, and `session-error`. Each includes monotonically increasing record index, UTC ISO time, and monotonic milliseconds since session start. Capture IDs increase when queued, including failures; never reuse them.
* Capture records include ID, reason(manual/trigger/timeout/evaluation-error), trigger ID or null, detection observation/verification or null, before/after observation or error, PNG relative path or null, start/end timestamps, stateEvidence, and error or null. Verification success and image confirmation are separate fields/concepts.
* session.json includes schemaVersion1, normalized URL, viewport/DPR, browser version, capture session ID, resolved module path and SHA256 or null, start/end timestamps, exit code, and primary/cleanup errors. It does not guess the application run, source binary, scenario hash, or log path; the acceptance report maps these externally.
* Apply the termination matrix and output-finalization order below. EOF is ignored in automated finishall mode; it must not cancel that run. Explicit quit remains available in either mode. Preserve primary errors and completed evidence during cleanup.
* A successful CLI exit establishes capture/check execution, not confirmed pixels. Required PNGs still need visual inspection before Subtask acceptance.

Test invalid result shapes, thrown verifiers, equality at deadline, repeated-trigger quit, blocked descendants, overflow, evaluation watchdog, and optional-module-set failure in addition to the focused cases above.

## Termination and preservation of completed evidence

The following matrix is authoritative for EOF, quit, and cancellation. Manual mode means no module or a module with `finish:'manual'`.

| Event | Mode | Required behavior | Exit result |
| --- | --- | --- | --- |
| stdin EOF | Automated `finish:'all'` | Close only the input reader; continue observation, deadlines, and capture until normal automatic completion | Determined by completed checks |
| stdin EOF | Manual | Stop observation/new requests and drain already queued captures; mark never-detected required once points incomplete | 0 if no failure/incomplete once points; otherwise1 |
| Explicit `quit` | Either | Same orderly shutdown as manual EOF; preserve completed/failed outcomes | 0 if no failure/incomplete once points; otherwise1 |
| All required once triggers terminal | Automated | Stop observation, finish queued diagnostics, then finalize output | 0 only if required checks/captures succeeded; otherwise1 |
| First SIGINT | Either | Stop observation/new requests; cancel queued captures; allow the single in-flight capture to finish within its existing5000ms limit. Mark pending once work cancelled without overwriting known failures |130 for otherwise successful cancellation; any run/output/cleanup failure requires1 |
| SIGINT during orderly draining | Either | Switch from draining to the SIGINT policy; cancel remaining queue | Same SIGINT result |
| Repeated SIGINT during cancellation | Either | Coalesce into the existing shutdown; do not start concurrent cleanup or exit before output finalization | Same result |
| Fatal browser/evaluation/output failure | Either | Stop observation; fail the affected operation; cancel work that can no longer run; best-effort output finalization |1 |

Already completed terminal records and saved PNGs remain unchanged. A known verification failure remains failed even if its pending diagnostic capture is cancelled.
For rearming triggers, preserve failure from any attempt; orderly shutdown with no pending request is completed. Interrupted pending work is cancelled unless already failed.
Neither the idle manual session nor waiting-to-rearm state has a session-wide timeout.

### Output-finalization order

Use one serialized append owner for results.jsonl. Append one UTF-8 JSON object plus newline per record and await each append before releasing its completion. Do not launch unawaited writes.
Open the file exclusively in the reserved directory and retain its file handle until finalization. This is an implementation choice within the existing result format, not a second logging subsystem.

1. Stop accepting new input and trigger-poll results according to the matrix. Stop trigger timers and prevent late callbacks from creating new capture requests. Allow before/after observations belonging to captures being drained or already in flight; they do not detect new trigger events.
2. Drain or cancel capture work as specified. Await the in-flight screenshot's existing timeout before closing its page unless the browser is already unusable. Successful screenshot completion produces a saved capture record; failure/cancellation never claims a usable PNG.
3. Record terminal outcomes for pending points and captures. Close the owned browser using the shared1000ms cleanup bound; record cleanup errors as session errors while the results file is still open.
4. Await every serialized append, then await closing the results file handle. No record may be appended after this step.
5. Write and await the final session.json, including end time, final exit code, primary error and cleanup errors. Write it only after the preceding results and cleanup outcomes are known.
6. Print the final summary from those same in-memory outcomes, after finalization succeeds or its failure is known. Set process.exitCode; do not use process.exit to abandon pending file writes.

On ordinary quit, manual EOF, and handled SIGINT, completed records must be complete parseable JSONL lines and previously saved PNGs must remain readable.
No fsync or crash-durability guarantee is required: power loss, forced process termination, OS failure, and externally deleted files are outside this promise.
On a filesystem failure, stop new work, preserve existing files without truncation or deletion, and report which output could not be finalized on stderr. Attempt remaining close/session operations where possible; retain the original error. Do not claim all records are parseable after a partial disk write.
If final session.json writing fails, report that explicitly and exit1; the summary must not describe it as a valid completed session file.

## Required negative and boundary validation matrix

Use `tools/capture/screenshots.test.mjs` for the capture rows below. These are specified acceptance cases, not new production features.
Use a temporary directory, stubbed browser operations, and fake time where appropriate. Use a local test page only for actual navigation and DOM behavior.

| Requirement | Input or discriminating case | Expected result | Test destination |
| --- | --- | --- | --- |
| CLI validation | Missing url/output-dir, missing flag value, duplicate flag, unknown flag, credential URL, unsupported URL protocol | Exit1 before browser creation; actionable error and not-created summary; no overwritten output | Capture argument tests |
| Output reservation | Existing session directory containing a sentinel PNG | Exit1; sentinel byte-for-byte unchanged; no browser created | Capture filesystem tests |
| Module validation | Missing default export/fields, empty automatic triggers, unsupported mode/repeat/finish, invalid ID/durations/args/function fields | Exit1 before browser creation; identify invalid field | Capture module tests |
| Dependencies | Duplicate IDs, unknown predecessor, A→B→A cycle | Reject with implicated IDs before browser creation | Capture module tests |
| Automatic completion | finishall plus rearm; valid once set with delayed diagnostic PNG | Reject rearm combination; valid run waits for diagnostic completion | Capture lifecycle tests |
| State versus transition | First observation true, then false, then true | State detects first true; transition detects only final true | Capture scheduler tests |
| Flight readiness | Loading/unavailable UI, then first valid altitude already inside band | Readiness detects; dependent transition does not treat initial loading as its false baseline | Flight example test in capture test file |
| Observed recovery | After readiness, valid band condition becomes false through lost availability and then true on recovery | Transition may detect recovery; evidence says UI condition became true, not physical altitude crossing | Flight example test in capture test file |
| Hold and rearm | True shorter than hold, false reset, sustained true; repeat true without false | Only sustained match detects; no repeated detection until false rearms | Capture scheduler tests |
| Deadlines and blocking | No match by deadline; prerequisite times out; match returned exactly at deadline | One timeout diagnostic attempt; descendants blocked; deadline wins equality | Capture scheduler tests |
| Invalid callback result | observe throws/returns Promise/null; verify throws or returns passfalse/check failure | Named failure with diagnostic attempt where browser responds; no successful trigger terminal | Capture predicate tests |
| Browser watchdog | Observe never returns | Fatal after1000ms; no queued evaluations; close browser; no fabricated diagnostic PNG | Capture watchdog test with stubbed evaluation |
| Queue behavior | Two independent detections during slow screenshot; exceed32 pending requests | FIFO with distinct IDs; observations continue; overflow recorded and exit1, no silent replacement | Capture queue tests |
| Image-state race | Trigger true at detection but false in capture bracket; missing bracket | Changed or unconfirmed with error; never automatic confirmed state | Capture bracket tests |
| Navigation | Initial HTTP redirect; later hash/history change; later new main document | First two allowed; new main document fails session; subframe navigation ignored | Capture real-browser fixture tests |
| Module/page failures | Missing expected module, wrong MIME/status, pageerror | Recorded session failure and exit1 with summary detail | Capture response tests |
| EOF distinction | Close stdin while automatic trigger still waiting; close it in manual mode | Automatic run continues and may complete; manual run drains and marks unfinished once points incomplete | Capture shutdown tests |
| Orderly quit | One saved capture, one in flight, one queued, one unmet once trigger | Drain captures, preserve PNGs and JSONL, unmet once point incomplete, final session and summary agree | Capture shutdown/filesystem test |
| SIGINT preservation | Same setup, then SIGINT; delayed append promise and repeated SIGINT | Queued capture cancelled; in-flight result awaited; saved PNG preserved; append/close/session writes awaited;130 when no run/output/cleanup failure exists | Capture shutdown/filesystem test |
| Error precedence | Earlier verification failure then SIGINT; browser close error; session.json write failure | Existing failure not hidden by130; cleanup retained separately; write failure exit1 and invalid-output notice | Capture failure/summary tests |
| Interrupted write safety | Stub results append to fail after earlier successful records | Stop new requests, retain earlier files, report incomplete output; no unsupported crash-durability claim | Capture filesystem failure test |
| Summary consistency | Successful run, failed check with saved PNG, timeout/blocked, pre-output failure, interrupted run | Counts/IDs/paths/status and exit agree with records; no pixel-confirmation claim | Capture summary tests |

Keep the retained shared-session tests in `tools/browser/session.test.mjs` for the added headless option and cleanup behavior.
The real-application scenario and lifecycle checks in Subtask4 validate actual DOM/producer behavior; these focused tool tests do not substitute for them.

## Final terminal summary and inspectable result fields

Print one final summary after the run result is known. Use stdout for normal completion and stderr for nonzero exit.
The summary must include these fields even when no PNG was saved:

* Absolute output directory and paths to `session.json` and `results.jsonl`; use `not created` when validation failed before reserving output.
* Capture request counts: requested, saved, failed, and cancelled. A saved diagnostic PNG counts as saved even when its verification failed.
* Trigger IDs grouped as completed, failed, timeout, blocked, incomplete, and cancelled. Print `none` for an empty group.
* Each failed named verification check with its trigger ID, actual value, and expected value. List session-level browser/module/output errors separately.
* The exit code and the statement `PNG content requires visual review; exit 0 does not confirm pixels.`

Use these exact JSON field names for records consumed by the documentation examples:

| Record | Required inspection fields |
| --- | --- |
| Every result record | `type` is one of the record types specified above |
| `capture` | `captureId`, `triggerId` or null, `status` (`saved`, `failed`, or `cancelled`), `pngPath` relative to output or null, `verification` or null, `stateEvidence`, `error` or null |
| `trigger-terminal` | `triggerId`, `status` (`completed`, `failed`, `timeout`, `blocked`, `incomplete`, or `cancelled`), `reason` or null |
| `session.json` | `exitCode` plus the provenance and lifecycle fields already specified above |

These names refine the existing records; they do not add another report format.
For a once trigger, `completed` requires successful verification and capture. Verification failure is `failed` even if its PNG is saved.
Timeout and blocked retain their specific terminal categories. Use incomplete for an orderly stop before a required point completes, and cancelled for interrupted pending work.
For a rearming trigger, emit one terminal result at session shutdown: failed if any attempt failed, otherwise completed when shutdown is orderly and no request remains pending. Capture records retain every attempt separately.

Illustrative failed-run summary:

```text
Output: C:\Work\msfs2024-artificial-horizon\.local\screenshots\demo-01
Session: ...\session.json
Results: ...\results.jsonl
Captures: requested=3 saved=3 failed=0 cancelled=0
Triggers: completed=target-up,level-up; failed=climb; timeout=none; blocked=none; incomplete=none; cancelled=none
Failed check: climb / trend painted / actual=false / expected=true
Exit: 1
PNG content requires visual review; exit 0 does not confirm pixels.
```

Add focused summary tests for a successful run, failed verification with a saved PNG, timeout with blocked descendants, pre-output argument failure, and an interrupted run.
Assert paths, counts, terminal IDs, named failures, and exit code agree with the recorded outcomes. Do not infer image confirmation from the summary.
