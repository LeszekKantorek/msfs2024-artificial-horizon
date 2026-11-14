# Subtask 1 analysis — Rendered sequence

## Authority and execution boundary

This is an implementation handoff for the matching Subtask in [the plan](browser-capture-evidence.md).
Source observations describe the inspected base `76174dfe6fb33eade3281034bf8ba54c3ea67f14`; selected changes below are implementation requirements, not claims of completed behavior.
Implement the specified behavior and run validation. Do not repeat discovery or choose a replacement design.
If the checkout differs materially or validation disproves an assumption, report the concrete conflict for plan correction. Do not silently invent a substitute.
No application changes or runtime validation were performed to prepare this analysis.

## Baseline prerequisite before edits

Before changing browser code, run the retained regression suite on the unchanged checkout and preserve its PNGs with revision and viewport identity in a separate ignored output directory.
After implementation compare normal (flag-disabled) captures to that baseline. The enabled footer is new behavior with no pre-feature equivalent; inspect its geometry and legibility rather than claiming an identical baseline allocation.
This prerequisite belongs to Subtask1 and must happen before its edits, not during Subtask5 documentation work.

## Verified data flow

| Source | Finding | Required edit |
| --- | --- | --- |
| [client.js](../web/telemetry/client.js), `publishSample` and `acceptSample` | Validation accepts monotonically increasing sequence; reconnect resets comparison to -1. `publishSample` drops sequence | Add `sequence: snapshot.sequence` to the `onSample` object. Keep `onAttitude` unchanged |
| [app.js](../web/app.js), initialization and `onSample` | App spreads sample into scheduler input and adds current status | Initialize status with `createStatusView(document, {showSequence: new URLSearchParams(location.search).get('showSequence') === 'true'})` |
| [panel.js](../web/panel.js), `composePanel.render` | All instruments render synchronously, then `status.render(frame)` runs | Preserve this order; do not add a second event listener or separate sequence timer |
| [status.js](../web/status.js), `render` and `invalidate` | Status owns availability and announcements | Add optional second argument `{showSequence=false}`. Render label from frame.sequence; clear on invalidate |
| [index.html](../web/index.html) and [styles.css](../web/styles.css) | Main currently has one flexible row; the instrument contains the live region | Add a hidden `p#capture-sequence` after the instrument section, outside the live region |

## Selected UI contract

* Disabled: label has `hidden`, empty text, and no layout space. Missing flag, `false`, and other values are disabled.
* Enabled: clear the label's hidden property at status-view initialization and set `main.dataset.showSequence='true'`; use rows `minmax(0,1fr) 18px`. Place the label in the second row with zero margin, 12px monospace text, and the existing foreground/background colors.
* Use `SEQ: <integer>` as text, including `SEQ: 0`. `data-sequence` stores the same integer for inspection.
* Before the first valid frame and after invalidation, show an empty label and remove `data-sequence`. Retain the diagnostic row while enabled to avoid geometry changes during outages.
* Do not announce each sequence, steal pointer events, or overlay instrument indications. The diagnostic row intentionally reduces panel height by 18px; compare enabled captures with enabled captures.
* Timed alert redraws reuse the last frame sequence. Sequence identifies the input sample, not the alert timer phase. Image inspection is still required for flashing.
* Preserve all other URL parameters. Do not change the SSE contract, logger, or server.

## Verification inputs and outcomes

| Check | Input and expected result |
| --- | --- |
| Client propagation | Extend [telemetry-client tests](../tests/telemetry-client.test.mjs): accepted samples 0 then 1 carry exact sequence; duplicate/out-of-order samples do not publish; reconnect permits restarted sequence |
| Scheduler/panel boundary | Extend [frame-scheduler tests](../tests/frame-scheduler.test.mjs) and [panel tests](../tests/panel.test.mjs) with explicit sequence-bearing frame cases: same frame reaches status after instrument render; timed redraw does not increment sequence |
| Browser label | Extend [retained regression entry](../tests/browser/pfd-browser.mjs), using existing fixture server: absent/false/true flag, changing accepted frames, source loss, expiry, recovery, resize |
| Painted layout | Capture enabled and disabled landscape/portrait using retained viewport cases. Inspect label legibility, no instrument overlap, and no layout jump on invalidation |

Use the existing browser fixture's generated sequence; do not add a runtime application testing hook.
Run `node --test tests/*.test.mjs`, rebuild the complete workspace, run `node tests/browser/pfd-browser.mjs`, and inspect the new PNGs.
