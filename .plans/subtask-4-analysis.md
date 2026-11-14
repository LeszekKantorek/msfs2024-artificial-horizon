# Subtask 4 analysis — Runtime assertion migration

## Authority and execution boundary


This is an implementation handoff for the matching Subtask in [the plan](browser-capture-evidence.md).
Source observations describe the inspected base `76174dfe6fb33eade3281034bf8ba54c3ea67f14`; selected changes below are implementation requirements, not claims of completed behavior.
Implement the specified behavior and run validation. Do not repeat discovery or choose a replacement design.
If the checkout differs materially or validation disproves an assumption, report the concrete conflict for plan correction. Do not silently invent a substitute.
No application changes or runtime validation were performed to prepare this analysis.


## Implementation reference

Use the [complete reference module and authoring steps](subtask-2-analysis.md#complete-reference-module) as the code pattern.
The tables and exact guard/boundary definitions below provide the remaining trigger-specific inputs, dependencies, and assertions.
The reference is complete for target-up; the other required triggers still need to be coded from this specified inventory.

## Deliverables and ownership

Use the generic entry point from Subtask 2. Add imported definition modules under `tests/browser/`:

* `test-scenario.triggers.mjs`: `finish:'all'`, complete short-pass conditions and module-set expectation.
* `source-lifecycle.triggers.mjs`: `finish:'all'`, live → externally stopped source → restarted Demo → unavailable SimConnect.
* `unavailable-source.triggers.mjs`: `finish:'all'`, fresh-client unavailable state and HTTP/browser diagnostics.

These files supply conditions, not additional executables. No process management or SSE listener belongs in them.
Use the optional `expectedModules: string[]` field specified in Subtask 2. At completion compare the sorted set of loaded JavaScript paths to it. Record missing/extra modules as run failure.

Use viewport568x240 and the diagnostic label consistently. DOM geometry expectations below remain the existing SVG-unit expectations; use layout-derived centers where specified.
All scenario triggers use `mode:'state'`, `repeat:'once'`, `holdMs:0`, `timeoutMs:300000` unless stated otherwise. This permits attachment mid-pass and capture on later passes. The case set can span passes; neither timestamps nor sequence identify a scenario-pass number.
All predicates read the actual rendered UI. Never read elapsed scenario time or SSE in a trigger.

## Existing assertions: complete migration inventory

References below are functions in [the old runtime script](../tests/browser/pfd-runtime-browser.mjs). Each grouped row lists the assertions to preserve.

| Source | Existing condition/assertions | Replacement |
| --- | --- | --- |
| `checkInitialDisplay` | Available instrument; no source badge or `.source`; no DEMO text in telemetry-status; slip-skid and turn available; altitude/baro consistency | `initial` UI trigger when available, heading354, selected altitude unavailable, VSI+4880. Verify all listed fields |
| `verifyAltitude` | Available altitude/baro; finite feet; drums=floor(abs(feet)/20)*20; minus sign iff negative; pressure matches number with2 decimals+inHg or STD; exact matching aria label | Shared pure verification called by initial, six digit-boundary captures, target initialization, and recovery |
| `checkInitialTelemetry` | UI heading354 and VSI+4880 | Initial UI trigger |
| `checkInitialTelemetry` snapshot assertions | No schema_version; source demo/state live; finite slip_skid, turn_rate_dps, IAS, GS, TAS, altitude, baro pressure, VSI, selected heading; heading354; no vne_kt | Add a focused assertion block to existing `tests/simconnect_cli.rs` using its `Application` and `wait_live_ias` infrastructure; wait for all required fields and initial heading before asserting. Source/profile properties belong in native/SSE integration, not UI |
| `checkRightTurnAndSpeeds` | Turn label3; world rotate(-30; IAS aria150/digits150; GS170; TAS160 and matching aria; overspeedfalse; airspeed trend hidden after history settles | `right-turn` matches turn3, bank30, and trend hidden. Verify speed values, labels, digits, and overspeed separately so incorrect speed values produce a failed check with a PNG. Capture replaces pfd-runtime-landscape.png |
| `checkNorthCrossing` | Heading aria0/selected0; displayed360; selected bug matrix.e equals half clip width within1e-4 | `north` during new one-second north hold; verify SVG center. Capture replaces pfd-runtime-heading-north.png |
| `checkSelectedAltitude` | Target10000/current8500; label/aria10000; alertnone; bug matrix.f14.5 | `target-up` UI condition reads altitude from aria and target text. Capture replaces selected-altitude PNG |
| `checkAltitudeHold` | Alt8500, VSI0, trend hidden | `level-up` after target-up detection; verify same observed snapshot; capture |
| `checkClimbAndTrend` | VSI1500, altitude8500≤h<8600, text+1500; trend x2,dy-60,predictionFt-h150, elementFromPoint at path midpoint hits path | `climb` after level-up; same geometry/paint assertions; capture replaces vertical-speed PNG |
| `checkSourceLoss` | Instrument unavailable, headingX, selected-heading bug has no transform, VSI unavailable, altitude trend hidden | Lifecycle `lost` transition after `live` detection; verify fields; capture |
| `checkRecovery` | Available; turn0; IAS/GS/TAS available; TAS=round(IAS*160/150); TAS text matches; IAS drums=floor(IAS); altitude/baro valid; heading354; selected altitude unavailable; VSI+4880 | Lifecycle `recovered` after lost. Match the initial-data combination before checking; same page, no reload |
| Failed-SimConnect block | source simconnect,state disconnected,transport connected,availablefalse; same loss assertions; six optional indicators hidden; overlay visible/text Source disconnected/opacity1 | Lifecycle `missing-source` after recovered; verify and capture |
| Failed-SimConnect snapshot | source simconnect,state disconnected,attitude null | Already covered by `Application::unavailable_stream_for` and `default_dll_is_beside_executable_and_explicit_missing_dll_does_not_fall_back` in tests/simconnect_cli.rs. Retain and run these; no duplicate assertion needed |
| Final reload block | New client unavailable; six optional nodes have local data-state unavailable | Separate fresh invocation with unavailable-source definitions, while missing-DLL application remains running. It is an explicit new-client test, not navigation per screenshot |
| `checkEmbeddedAssets` loaded module set | Exact imported paths | expectedModules list below; browser response listener also checks200/JavaScript MIME |
| HEAD assertions | vsi,trend,heading,true-airspeed return200,JavaScript MIME,empty body | `tests/http.rs` already covers first three in `compact_vsi_altitude_trend_and_heading_assets_support_get_and_head`; add true-airspeed/createTrueAirspeed to that table |
| Obsolete routes | math,svg,telemetry-client,airspeed,horizon,fixed-symbols,airspeed/calculations,horizon/calculations,vertical-speed return404 | Existing `static_router_does_not_create_a_telemetry_source` covers first eight; compact VSI test covers vertical-speed. Keep both; no duplicate browser fetch needed |
| Health/process assertion | Health returns status ok even with unavailable source; process remains alive | Existing Application health probe in native integration plus explicit health check during external setup below |
| Page errors | None throughout run | Generic capture records pageerror and fails the run |
| Cleanup | Browser/process cleanup retains primary error | Browser cleanup in generic CLI; operator owns externally started process. Native integration retains Application cleanup |

The six optional nodes are `#airspeed`, `#ground-speed`, `#true-airspeed`, `#altitude`, `#barometric-setting`, `#selected-altitude`.
Use computed visibility for existing nodes during loss; fresh-client local state is a separate check because invalidation masks retained DOM.

### Expected imported module paths

Carry this exact inspected set into `expectedModules`; the planned sequence change adds no browser module:

```javascript
[
'/airspeed/ground-speed.js', '/airspeed/true-airspeed.js', '/airspeed/helpers.js', '/airspeed/ias.js', '/airspeed/index.js',
    '/airspeed/readout.js', '/airspeed/references.js', '/airspeed/tape.js', '/altitude/baro.js', '/altitude/index.js', '/altitude/readout.js', '/altitude/selected.js', '/altitude/tape.js', '/altitude/trend.js', '/altitude/vsi.js', '/app.js', '/fixed-symbols/aircraft.js', '/fixed-symbols/bank-scale.js', '/fixed-symbols/index.js', '/frame-scheduler.js', '/heading.js', '/helpers.js', '/horizon/bank-pointer.js', '/horizon/index.js', '/horizon/pitch-scale.js', '/horizon/world.js', '/layout.js',
    '/panel.js', '/slip-skid.js', '/status.js', '/telemetry/client.js', '/telemetry/validation.js', '/turn-rate.js',
]
```

## Additional full-scenario UI cases

Use selected altitude text to distinguish directions; never infer direction from wall-clock timing.

| IDs | Activation and condition | Verification |
| --- | --- | --- |
| `digits-zero-up`, `digits-1000-up`, `digits-10000-up` | Paired before/after points defined below; selected target unavailable, VSI positive | Verify both sides and compare sign/drums; two captures per boundary |
| `digits-10000-down`, `digits-1000-down`, `digits-zero-down` | Paired before/after points below; selected target unavailable, VSI negative | Verify both sides and compare sign/drums; two captures per boundary |
| `approach-up-1000`, `approach-down-1000` | After corresponding target initialization; target10000/1000; data-alert within-1000 and flashingtrue | Text target, turquoise fill, bug visible; PNG |
| `approach-up-1000-done`, `approach-down-1000-done` | After corresponding flash detection; data-alertnone and flashingfalse while absolute error between200 and1000 | Text visible, no yellow; PNG |
| `approach-up-200`, `approach-down-200` | After corresponding1000-done; data-alert within-200 and flashingtrue | Same target, visible bug, turquoise; PNG |
| `approach-up-200-done`, `approach-down-200-done` | After corresponding200 detection; alertnone/flashingfalse and error≤200 | Text visible, captured band; PNG |
| `deviation-up`, `deviation-down` | After corresponding200-done; alertdeviation/flashingtrue | Yellow value, turquoise bug, and aria deviation; PNG |
| `deviation-up-done`, `deviation-down-done` | After corresponding deviation detection; alertdeviation/flashingfalse | Steady visible yellow value; visible turquoise bug; PNG |
| `target-down` | After deviation-up-done; target1000, altitude2500, VSI0 | Target change clears yellow value and history, target label1000, trend hidden; PNG |
| `descent` | After target-down; VSI-1500, altitude2400<h≤2500 | Text-1500; trend x2,dy+60,prediction change-150, midpoint painted; PNG |

Read colors from computed styles: selected value uses --speed-reference normally and --speed-yellow during deviation; the bug always uses --speed-reference. Compare resolved colors, not invented RGB constants. This follows web/styles.css lines115–120.
For each approach/deviation flash, also register a dependent `-hidden` point when selected value visibility is hidden, and a subsequent `-visible` point when visible while flashing remains true.
Keep the bug visible during both phases. Capture-state uncertainty remains explicit; a pass requires inspection of the required image evidence, not only predicate success.
Dependencies use detection events, so an unsuccessful screenshot does not stop later observations. Verification failures still fail the run.
The initial/boundary and independent right-turn/north conditions observe concurrently. A 300s timeout is a failure bound, not a screenshot timer.

## External setup and source-contract tests

Run the existing `tools/live/run.mjs` in terminal1. It owns the application and writes raw SSE snapshots. Run the independent screenshot CLI in terminal2.
The application's `--log` journal is optional additional native diagnostics: `journal.rs` currently writes schema4 and does not journal every publication. It cannot substitute for the raw SSE recording when looking up an arbitrary visible sequence.

```powershell
New-Item -ItemType Directory -Force .local/live | Out-Null
$appExe = Join-Path $PWD 'target/x86_64-pc-windows-msvc/release/msfs2024-artificial-horizon.exe'
node tools/live/run.mjs --exe "$appExe" --url http://127.0.0.1:8080/api/v1/events --output .local/live/demo-01.jsonl --duration-seconds 600 -- --source demo.dll --scenario demo-scenarios/test-scenario.jsonl --listen-address 127.0.0.1 --port 8080
```

Use a new telemetry filename per application launch. A600s recording window covers the expected healthy capture run after mid-pass attachment without the old1000-message cutoff. It is not a guarantee that a failing trigger finishes before the logger stops; if the limit ends first, record the capture run as incomplete and stop it explicitly. The capture tool neither coordinates nor owns the recorder.
Keep the paired capture directory, raw recording path, scenario SHA256, application/DLL SHA256, and launch arguments in the acceptance report.
Separate SSE subscribers may skip different publications; an absent exact sequence is an explicit unmatched result, not permission to substitute the nearest record.
For lifecycle verification:

1. Start the recorder as above, then capture with source-lifecycle definitions; wait for the saved live PNG before stopping anything.
2. Ctrl+C the recorder in terminal1 and wait for its owned application to exit. Wait for the lost PNG; relaunch the same recorder command with a new raw telemetry filename at the same address.
3. Wait for recovered PNG; stop the recorder and wait for process exit. Relaunch it with a new telemetry filename and application arguments `--source <absolute-nonexistent-SimConnect.dll> --listen-address 127.0.0.1 --port 8080` (no scenario).
4. Wait for missing-source capture; verify `Invoke-RestMethod http://127.0.0.1:8080/health` returns status ok.
5. Run a separate capture command with unavailable-source definitions to test fresh-client behavior. Stop its owning recorder when finished.

Lifecycle triggers use300000ms deadlines to allow operator actions. Do not add a coordinator or fake UI transitions.
For native tests, use existing `tests/simconnect_cli.rs` Application helpers for process setup and SSE reads. These are retained test infrastructure, not new capture orchestration.

## Removal gate

Delete `pfd-runtime-browser.mjs` only after the inventory above has replacements implemented and their checks pass.
Run all required scenario triggers, inspect PNGs, run native/HTTP regressions, and verify source-loss/recovery with the same page.
Report each missed condition, failed assertion, or unconfirmed required image. Do not call a missed point covered merely because the old script was removed.

## Lifecycle trigger definitions

* `live`: state/once, no dependencies; match available Demo with initial heading354, selected target unavailable, and VSI+4880. Verify the initial-display fields.
* `lost`: transition/once after live; require an observed available state before unavailable. Verify source-loss fields. Operator waits for this PNG before restart.
* `recovered`: state/once after lost; match the initial-data recovery combination listed in the inventory. Verify every recovery assertion.
* `missing-source`: state/once after recovered; match source simconnect, transport connected, state disconnected, availablefalse. Verify overlay and optional-node visibility.
* Separate unavailable-source module: one state/once trigger `fresh-unavailable` with the same missing-source match plus all six optional nodes' data-state unavailable.

All lifecycle triggers use timeout300000ms and holdMs0. Dependencies fire on detection; operator waits for the relevant saved PNG before each next process action.

## Required guards and boundary pairs

Only live-reading scenario cases, `live`, and `recovered` require `.instrument[data-available="true"]` plus available required indications. Do not apply that guard to `lost`, `missing-source`, or `fresh-unavailable`; their guards are specified below.
For target/alert chains, require the exact target throughout: up10000, down1000, including `-done`, `-hidden`, and `-visible` children.
Require the same alert kind while detecting hidden/visible phases. For a done point also require its numeric error band and previous flash detection; invalidation cannot complete it.
`target-up` matches target10000/current8500; verify alertnone instead of making that expected reset part of detection.
`target-down` matches target1000/current2500/VSI0 after deviation-up-done; verify alertnone/flashingfalse, ordinary value color, and hidden trend.

Expand each of the six boundary IDs into `-before` and `-after`. After depends on its matching before detection. Direction is the sign of the raw VSI value from its aria label.

| Boundary | Before band | After band | Expected distinction |
| --- | --- | --- | --- |
| zero-up | -120≤alt<0 | 0≤alt≤120 | Minus sign disappears |
| 1000-up |980≤alt<1000 |1000≤alt≤1020 | Thousands column becomes1 |
| 10000-up |9980≤alt<10000 |10000≤alt≤10020 | Ten-thousands column becomes1 |
| 10000-down |10000≤alt≤10020 |9980≤alt<10000 | Ten-thousands column becomes empty |
| 1000-down |1000≤alt≤1020 |980≤alt<1000 | Thousands column becomes empty |
| zero-down |0≤alt≤120 |-120≤alt<0 | Minus sign appears |

Down-before also depends on the corresponding up-after detection. Verify the existing floor(abs(alt)/20)*20 drum rule in every observation.
Keep numeric bounds inclusive/exclusive as shown. Each pair has two PNGs, subject to capture-state inspection; a single pre-boundary PNG does not establish a transition.

For alert-done stages use200<error≤1000 for1000-done, error≤200 for200-done, and error>200 plus data-alertdeviation for deviation-done.
Bug visibility means not displaynone, nonzero geometry, and its expected clip position; selected readout visibility is tested separately.

### Availability guards by case

| Case family | Required guard | Values assessed separately |
| --- | --- | --- |
| Live scenario readings and alerts | Instrument availabletrue; every indication used for numeric matching has data-stateavailable (slip/turn use data-availabletrue) | Expected values, color, text, and geometry belong in verify unless they identify the triggering state |
| `live` and `recovered` | Instrument sourcedemo, availabletrue; readable initial heading/VSI/target availability combination from their definitions | Initial/recovery checks from the migration inventory |
| `lost` | After live, observe instrument availabletrue as the false baseline, then availablefalse as the true loss condition | Do not require specific source/transport/reason: transport failure and later freshness invalidation can occur in either order. Verify X/hidden/unavailable fields |
| `missing-source` | Instrument sourcesimconnect, transportconnected, statedisconnected, availablefalse | Overlay message/opacity, optional visibility, and loss fields |
| `fresh-unavailable` | Same source/transport/state/availability guard as missing-source in the separately opened page | Additionally require each of the six optional nodes to have local data-stateunavailable in verification |

All cases guard missing DOM elements and return matchesfalse with missing selectors until their required nodes exist. A missing node must not become numeric zero.
The common live-reading guard does not suppress lifecycle failure observations.

## UI observation field map

This map specifies reads for every UI field in the migration inventory and added cases. It is documentation for the definition modules, not a new runtime selector registry.
Sources: [runtime assertions](../tests/browser/pfd-runtime-browser.mjs), [heading](../web/heading.js), [altitude readout](../web/altitude/readout.js), [selected altitude](../web/altitude/selected.js), [VSI](../web/altitude/vsi.js), and [styles](../web/styles.css).

For labelled numbers, match the complete literal label shape below, extract its numeric token, and require a finite Number. Use null on absent/malformed/unavailable labels; do not use Number(null) or Number('') as a valid reading.
Use the same synchronous observe call for all related fields. Units in geometry rows are local SVG/CSS-pixel coordinates for the fixed viewport unless screen conversion is stated.

| Field | Selector and exact read | Unit / expected use |
| --- | --- | --- |
| Global state | `.instrument`: dataset.source, transport, state, available | Strings demo/simconnect; connected or other transport; live/disconnected etc.; available is literal true/false string |
| Removed source labels | `document.querySelectorAll('#source-badge, .source').length`; `#telemetry-status`.textContent | Count0; text must not contain DEMO |
| Slip/turn availability | `#slip-skid`, `#turn-rate`: dataset.available | Literal true for live initial checks |
| Turn rate | `#turn-rate` aria-label `Turn rate <number> degrees per second` | degrees/s;3 for right-turn,0 for initial recovery |
| World bank transform | `#world-rotation`.getAttribute('transform') | Right-bank case starts with `rotate(-30 `; do not interpret the SVG angle as the opposite UI bank sign |
| IAS state/value | `#airspeed`: dataset.state, aria-label `Indicated airspeed <number> knots`, dataset.overspeed | knots;150 and overspeedfalse in right-turn; finite value in recovery |
| IAS drums | `#airspeed-digits > g`, each firstElementChild.textContent, joined in DOM order | String150 for held150; String(floor(IAS)) during recovery |
| GS | `#ground-speed`.dataset.state; `#ground-speed-value`.textContent | Available; displayed integer knots170 at right-turn |
| TAS | `#true-airspeed`.dataset.state and aria-label `True airspeed <number> knots`; `#true-airspeed-value`.textContent | Display-rounded knots160 at right-turn; round(IAS*160/150) during recovery |
| Airspeed trend | `#airspeed-trend`: getComputedStyle(node).display | none for settled speed; missing node is not a hidden trend |
| Heading and target | `#heading`.dataset.state and aria-label `Magnetic heading <number> degrees, selected heading <number> degrees` | Degrees; north both0. Initial/recovery may instead have selected-heading unavailable; read heading separately from that label suffix |
| Heading text | `#heading-value`.textContent |354 initial/recovery;360 at normalized north;X during source loss |
| Heading bug and center | `#selected-heading-bug`: transform.baseVal.consolidate()?.matrix.e; `#heading-tape-clip rect`.getBBox().width/2 | Absolute difference<1e-4 at north. For loss require getAttribute('transform')===null rather than converting missing transform to0 |
| Altitude state/value | `#altitude`.dataset.state and aria-label `Barometric altitude <number> feet` | Unrounded feet; case bounds in tables above |
| Altitude sign | `#altitude-sign`.textContent | Unicode minus U+2212 iff altitude<0; otherwise empty |
| Altitude drum value | `#altitude-digits > g`, firstElementChild.textContent per group, join then Number | floor(abs(altitude)/20)*20; four groups required |
| Altitude individual columns | Same four groups in order: index0 ten-thousands,index1 thousands,index2 hundreds,index3 final two-digit20ft drum | Read firstElementChild.textContent.1000 crossing changes index1 between empty and1;10000 crossing changes index0 between empty and1 |
| Barometric state/value | `#barometric-setting`.dataset.state; `#barometric-value`.textContent; group aria-label | Available; text is digits+dot+two digits+` inHg` or STD; aria is `Barometric setting <text>, click or tap to switch display` |
| VSI state/raw value | `#vertical-speed`.dataset.state; aria-label `Vertical speed <number> feet per minute` | ft/min; raw sign selects boundary direction;1500/-1500 for trend cases;0 for holds |
| VSI displayed text | `#vertical-speed-value`.textContent and computed visibility |+4880 initial/recovery; +1500 climb; -1500 descent;0 hold; X unavailable |
| Altitude trend presence/prediction | `#altitude-trend`: computed display, dataset.predictionFt parsed as finite Number only when attribute exists | none during hold/loss; prediction minus observed altitude is150 or-150 ft |
| Altitude trend geometry | Same path: getPointAtLength(0), getPointAtLength(getTotalLength()) | x2; end.y-start.y=-60 for climb,+60 for descent |
| Altitude trend painted point | DOMPoint(start.x,(start.y+end.y)/2).matrixTransform(path.getScreenCTM()); document.elementFromPoint(screenX,screenY) | Returned node must equal the trend path. Null matrix/path data is a failed check, not a valid point |
| Selected altitude | `#selected-altitude`: dataset.state; `#selected-altitude-value`.textContent; group aria-label `Selected altitude <number> feet` optionally followed by `, deviation` | Target10000 up,1000 down; numeric error uses the unrounded number in aria, not rounded text |
| Alert state and phase | `#selected-altitude`: dataset.alert, dataset.flashing; selected value computed visibility | Alert none/within-1000/within-200/deviation; flashing literal true/false; hidden or visible phase |
| Selected value color | `#selected-altitude-value`: computed fill | Resolve --speed-reference normally, --speed-yellow during deviation. Current CSS values are #86def5 and #ffdd43; normalize RGB representations before comparison |
| Selected altitude bug | `#selected-altitude-bug`: computed display, visibility, fill; getBBox(); consolidated transform.matrix.f | Not displaynone, visibilityvisible, nonzero bounds; ordinary turquoise fill even on deviation. Upper clamp14.5 at target-up |
| Bug clamp geometry | `#altitude-tape-clip rect`.getBBox().height givesH; current altitude h and raw target t from labels | Expected local y=clamp(H/2+(h-t)*0.4,14.5,H-14.5). Use1e-4 tolerance for computed positions; do not import product positioning code into checks |
| Optional visibility on loss | Six selectors listed after the inventory: computed visibility for each | hidden on unavailable UI; local data-state may retain prior values until fresh render |
| Fresh optional state | Same six selectors: dataset.state | unavailable for separate fresh-unavailable test |
| Unavailable overlay | `#unavailable`: textContent and computed visibility; `.unavailable-panel`: display, visibility, opacity | Source disconnected; shown (not displaynone, visibilityvisible), opacity1 |
| Sequence label (added by Subtask1) | `#capture-sequence`: textContent and data-sequence | SEQ: integer; clear on invalidation. Read final lookup sequence from PNG during review, not from a later DOM observation |

Missing raw/native fields in the SSE rows (schema_version, vne_kt, pressure_hpa and raw optional availability) intentionally have no UI selector. Their destination remains the native/SSE tests specified in the inventory.
For numeric comparisons use the stated case tolerance; no default rounding may change a boundary's inclusive/exclusive meaning.

## Selecting checks after an instrument change

Run the retained `node tests/browser/pfd-browser.mjs` entry point for a UI change. It has no per-instrument CLI filter in this plan.
Use the validation scope matrix for native/unit checks and whether compiled-application validation applies.
When runtime validation applies, the acceptance run uses the full `test-scenario.triggers.mjs` case set. Smaller sets below are for focused development and rerunning failures; they do not replace required final coverage.

| Changed area | Focused runtime trigger IDs |
| --- | --- |
| Attitude, bank, slip/skid, turn | `initial`, `right-turn` |
| IAS, GS, TAS, airspeed trend | `initial`, `right-turn` |
| Heading or selected heading | `initial`, `north` |
| Altitude digits or barometric setting | `initial`, all twelve `digits-*-before`/`digits-*-after` points, `target-up`, `level-up`, `climb`, `target-down`, `descent` |
| Selected altitude and alerts | `target-up`, `target-down`, all approach/deviation points listed below |
| VSI or altitude trend | `initial`, `level-up`, `climb`, `descent` |
| Shared frame/status/availability, sequence label, layout, capture mechanism, or scenario data | Full scenario set; also the source-lifecycle and fresh-unavailable modules for affected lifecycle behavior |

The twelve digit points expand the six boundary IDs already specified, each with before and after.
The alert set contains `approach-{up,down}-{1000,200}` and `deviation-{up,down}`, each with its base, `-hidden`, `-visible`, and `-done` IDs.
Always include transitive `after` predecessors, even when they belong to another instrument or an earlier phase. This is required for history-dependent checks.
Run lifecycle modules as a whole when changing source loss/recovery: their externally induced sequence is part of the test.

### Choosing the kind of change

| Needed evidence or missing capability | Required action |
| --- | --- |
| Isolated formatting, geometry, boundary, invalid-input, or controlled-clock behavior | Extend the retained regression/unit checks. Use controlled fixtures; a runtime screenshot alone does not cover these combinations |
| The real UI can already reach the required state with current Demo data or the current flight | Add or adjust the UI trigger and its separate verification; keep scenario data unchanged |
| A required native/runtime input sequence cannot occur with existing Demo data | Change the scenario and its independent native checkpoints, then add/adjust corresponding UI triggers. Do not add screenshot markers or application hooks |
| A display change affects both deterministic presentation and real integration | Update both the focused regression and runtime trigger checks; select full acceptance coverage through the existing validation matrix |
| Only screenshot timing/diagnostics fail while the UI state is already reachable | Correct the capture definition or tool under its existing contract, not the producer scenario merely to hide a capture failure |

Apply the existing instrument grouping and dependency-preserving rerun procedure below after selecting the change type. Focused iterations do not reduce final acceptance coverage.

## Rerunning one failed case without a new CLI option

Save this module as `.local/triggers/retry-climb.mjs` after Subtask4 has created the committed scenario definitions.
Change only `requested` to the failed ID or selected IDs from the table above. It retains the original functions, arguments, trigger modes, and module expectations.

```javascript
import suite from '../../tests/browser/test-scenario.triggers.mjs';

const requested = ['climb'];
const byId = new Map(suite.triggers.map(trigger => [trigger.id, trigger]));
const included = new Set();
function include(id) {
  const trigger = byId.get(id);
  if (!trigger) throw new Error(`Unknown trigger ${id}`);
  if (included.has(id)) return;
  included.add(id);
  for (const predecessor of trigger.after) include(predecessor);
}
for (const id of requested) include(id);
const triggers = suite.triggers.filter(trigger => included.has(trigger.id));
if (triggers.some(trigger => trigger.repeat !== 'once')) {
  throw new Error('This automatic rerun example requires once triggers');
}
export default { ...suite, finish: 'all', triggers };
```

```powershell
node tools/capture/screenshots.mjs --url "http://127.0.0.1:8080/?showSequence=true" --output-dir .local/screenshots/retry-climb-01/ --triggers .local/triggers/retry-climb.mjs
```

For climb, the selected set includes target-up and level-up before climb. Keep expectedModules through the spread of suite.
The new capture session reobserves those UI events, possibly on a later Demo pass. It does not replay results, inherit history from the previous browser page, or mark prerequisites satisfied from old evidence.
Do not strip dependencies to make a case fire immediately. A lifecycle rerun requires repeating its external setup; a real-flight event may require a new opportunity during flight.
Retain the original failed output and use a new output directory. Report focused results as partial coverage until the required full acceptance checks run.

## Provenance for trigger imports

The entry module SHA256 in session.json does not identify imported code. Record a dependency manifest in the acceptance report alongside the existing application/DLL/scenario identity.
For each run, list the resolved path and SHA256 of the entry module and every transitively imported local JavaScript or data file that can affect its definitions or verification.
This includes a rerun wrapper, the committed suite it imports, and helpers imported by that suite. Include files loaded through explicit fs reads or dynamic import when used; identify those paths in the module's authoring notes before the run.
Use the same source files throughout one run. Do not edit imported helpers while capture is active.

Use `Get-FileHash -Algorithm SHA256 -LiteralPath <path>` for each listed file. Record the Git revision and whether those files have local changes; a commit alone does not identify uncommitted helpers.
Node built-ins need no file hash; record the Node version. If an external package is used, record its resolved version and the lockfile hash in addition to local files.
No automatic dependency scanner or new artifact subsystem is required. The author supplies the explicit dependency list; incomplete provenance must be reported as incomplete.
Subtask5 publishes this rule with the rerun example so imported definitions are not mistaken for a self-contained entry file.
