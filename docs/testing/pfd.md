# Shared EFI validation

Controlled browser inputs, desktop commands, layout checks, and full-panel acceptance.

## Controlled browser fixtures

1. Start the development-only fixture server.

   ```powershell
   node tests/browser/helpers/browser-fixture-server.mjs
   ```

2. Open `http://127.0.0.1:8082`.
3. Enter one JSON line at a time in the server terminal.
4. Inspect the result after each line.

```json
{"state":"live","hold":false,"malformed":false}
{"state":"paused"}
{"state":"disconnected"}
{"state":"invalid"}
{"state":"waiting"}
{"state":"stale"}
{"state":"live","hold":false}
{"hold":true}
{"hold":false,"malformed":true}
{"malformed":false}

```

| Scenario | Check |
| --- | --- |
| `hold: true` | Comments continue without samples. Live attitude becomes stale after one second |
| Malformed JSON | Instrument becomes obscured |
| Reconnect or background/resume | Check the logged active SSE count for parallel subscriptions from one page |

> The fixture server serves real web assets with synthetic SSE on loopback by default. Explicit LAN binding supports phone checks. It is separate from the release binary.
> These checks establish UI behavior, not simulator compatibility.

Use each [feature's controlled inputs](../efi/index.md#features) for instrument-specific cases.

## EFI presentation validation procedure

Use [project-validate-presentation](../../.agents/skills/project-validate-presentation/SKILL.md) for the presentation validation procedure.
The skill owns its steps. This page retains shared check commands and panel criteria.
Feature pages own instrument criteria; [device procedures](devices.md) and [evidence rules](evidence.md) remain shared.

## Responsive EFI layout and desktop checks

Open `http://127.0.0.1:8082/layout.html` for the development-only arrangement of all 17 intended illustration items.
The `LAYOUT FIXTURE` badge distinguishes the development fixture from the compiled demo.
Use this page before changing layout geometry or adding the next EFI instrument.
No fixture page or fixture control script enters the Rust binary.

| Check | Expected result |
| --- | --- |
| 320x240, 326x246, 320x480, 390x664, 568x240, 667x280, 844x320 CSS px | No scrolling or indication-content collisions, with readable central attitude and supplemental values |
| Minimum usable area | 320 CSS px wide and 240 CSS px high after browser bars/safe areas |
| Rotation / viewport changes during streaming | Preserve values and sample deadline, with a round ball and undistorted symbols |

The scripted desktop checks use Playwright with an installed browser.
They are required for the EFI changes in the [validation matrix](index.md#validation-by-change-scope) and optional for unrelated exploration.
It adds no application dependency or frontend build step.

Browser tests are grouped under `tests/browser/`:

* `pfd-browser.mjs` runs controlled fixture checks.
* `pfd-runtime-browser.mjs` checks the compiled application.
* `checks/` contains instrument scenarios called by `pfd-browser.mjs`.
* `helpers/` contains the fixture server and layout fixture module.

Shared input data remains in `tests/fixtures/` for both Rust and JavaScript tests.
Run the browser scripts separately from the local Node `tests/*.test.mjs` suite.
Run all commands below from the repository root.

Install the test tool in ignored local storage if it is not already available:

```powershell
npm install --prefix .local/browser --no-save playwright
$env:PLAYWRIGHT_MODULE = Join-Path $PWD '.local/browser/node_modules/playwright'
$env:BROWSER_CHANNEL = 'msedge'
node tests/browser/pfd-browser.mjs
node tests/browser/pfd-runtime-browser.mjs target/x86_64-pc-windows-msvc/release/msfs2024-artificial-horizon.exe
```

| Script | Input | Checks | Output |
| --- | --- | --- | --- |
| `pfd-browser.mjs` | Real web assets and isolated loopback fixture on an OS-assigned port | Viewports, poses, partial/old snapshots, stale states, resize, lifecycle, identity, frozen frames, hierarchy/paint order, shared clips, and background coverage | Screenshots under `.local/pfd-browser/`, including opacity candidates in both orientations against sky and ground |
| `pfd-runtime-browser.mjs` | Release binary built from the checked source and passed explicitly | Every current imported module with HTTP 200/JavaScript MIME, Rust demo SSE, coherent bank/turn state within 25 seconds, and restart recovery without reload | `.local/pfd-runtime-landscape.png` |

The scripted geometry checks include these regression cases:

* The coverage check must reject a deliberately narrowed background clip and pass after restoration.
* The 326x246 viewport provides a 320x240 usable panel with the normal 3-pixel margins.
* Import changes require matching Rust embedded routes, fixture allowlist/MIME, HTTP asset tests, and the runtime expected-module set.
* Check the shared panel attitude clip used by the bank pointer and fixed symbols, including resize while data is expired.

Asset checks cover the current module graph:

* The [EFI composition guide](../efi/composition.md#component-responsibilities) describes runtime ownership. The [feature directory](../efi/index.md#features) identifies implementing modules.
* `src/http/assets.rs` registers the current modules, HTML, and CSS through explicit mappings.
* `tests/http.rs` checks the explicit embedded routes and JavaScript MIME.
* Both browser scripts check retired routes return 404 without compatibility facades.

Shared retired helpers map to the following owner.
Feature pages list their retired routes; both browser scripts must check those routes return 404:

| Retired route | Current owner |
| --- | --- |
| `/math.js`, `/svg.js` | `/helpers.js` |

Pure checks follow the existing module boundaries:

* Pure functions remain testable from their visual modules in Node without DOM.
* Separate calculation files are not required for tests.
* Check observable boundaries, carry behavior, references, warnings, clipping, and recovery instead of duplicating private implementation formulas.

> Review the screenshots: geometry checks alone do not establish readability.
> Synthetic padding and page events do not establish actual cutout, browser-bar, or background behavior on phones.

## Planned EFI demo acceptance

> The procedures apply to the EFI demo and do not establish passing results or live EFI compatibility.

* Each feature issue owns its automated and real-phone evidence.
* Use the [device evidence template](devices.md#real-phone-acceptance-procedure).
* Preserve original simulator acceptance scope.

| Area | Scenarios and acceptance |
| --- | --- |
| End-to-end values | Independent fixtures agree across typed Rust data, deterministic demo, SSE, and display. Clients share values |
| Compatibility | Old v1 snapshots display attitude. Missing EFI fields display unavailable, never zero. No excluded-instrument placeholders |
| Instrument failures | Missing/invalid optional values affect only their indication. Source/transport loss invalidates the panel |
| Timing | Controlled clocks check shared freshness and expiry before pending draw. No wall-clock sleeps |
| Instrument geometry | Apply all [feature scenarios](../efi/index.md#features) during full-panel acceptance |
| Mobile layout | From 320 CSS px, portrait, short landscape, safe areas, browser-bar changes, rotation while streaming |
| Layout acceptance | No scrolling, indication-content collisions, clipped warnings, or distorted symbols, with intentional background overlap allowed |
| Lifecycle | Wi-Fi recovery, server restart, lock/background/resume, source pause/stale/invalid states, partial failures |
| Recovery | Fresh sample before valid readings return. No duplicate subscription |

1. In #20, inspect layout fixtures for the complete intended arrangement before later slices add live demo values.
2. Check real-phone readability with each added instrument.
3. Run the [automated checks](automated.md#automated-checks) and follow [CONTRIBUTING](../../CONTRIBUTING.md).
4. Update the feature's contract fixtures and procedures in the same PR.

> Desktop viewport checks do not establish mobile acceptance.

## Modular and layered EFI acceptance

These checks cover instrument composition, layers, and visual objects:

* #36 implements modular rendering.
* #37 implements layer containers and full-panel background coverage.
* #56 composes visual objects and keeps object-specific calculations with their implementations under [ADR 0005](../adr/0005-visual-object-composition.md).
* Opacity acceptance requires real-phone results.
* Run the [automated checks](automated.md#automated-checks) for each implementation PR.
* Record real iOS Safari and Android Chrome results separately from desktop evidence.

> This document defines procedures, not passing results.

| Owner | Checks |
| --- | --- |
| #36: lifecycle | Coalesce latest samples, preserve deadlines through resize, cancel invalid work, and renew freshness for unchanged fresh values |
| #36: recovery | Expire without new samples, reject an expired sample before paint, and recover after reconnect and resume only with fresh data |
| #36: assets | Serve every imported module through the compiled Rust server and the development fixture server |
| #36: DOM ownership | Preserve SVG node identity across samples and resize. Check actual transformed geometry, not only transform strings |
| #37: layers | Fill the entire panel with sky/ground, clip pitch markings centrally, and preserve fixed references and layer order |
| #37: readability | Check digits, ticks, and value windows against sky and ground, and record the selected background opacity |
| #37: bounds | Permit intentional background overlap, reject content collisions, and include painted strokes in bounds checks |
| #37: invalidation | Obscure the expanded background on global loss, keeping optional failures local |
| #56: ownership | Same complete read-only frame at instrument entry points, concrete child inputs, immediate-parent shared hosts/clips, and no visual frame cache/timer |
| #56: composition | Preserve hierarchy, defs/paint order, SVG identity, ARIA, common attitude clip, and coherent once-per-frame transforms |
| #56: assets | Verify the complete module graph through explicit Rust/fixture registration and HTTP/MIME checks, with the listed retired routes returning 404 |

Record refactor validation against the final revision:

1. Preserve baseline screenshots.
2. Record the tested commit, environment, complete logs, native exits, and [build provenance](evidence.md#build-provenance).
3. Describe and assess each [screenshot difference](evidence.md#screenshot-comparisons) without inventing an unverified cause.
4. Run required final checks on the combined revision, following the [evidence reuse rules](index.md#evidence-reuse).
5. Keep actual iOS Safari and Android Chrome acceptance unchecked until passing device evidence exists.
6. Record remaining checks in the owning issue or a linked validation record.

Apply the [shared phone procedure](devices.md#real-phone-acceptance-procedure) for #37 and compare opacity with [phone fixtures](devices.md#phone-fixture-setup):

1. Open the full 17-element development fixture.
2. Compare `/layout.html?opacity=0.50`, `0.65`, and `0.80` against sky and ground.
3. Record the selected value and real-device evidence in #37.
4. Include extreme poses in the shared layout scenarios.
5. Repeat lifecycle checks after geometry changes without renewing the sample deadline.

Later feature slices extend these checks:

* Keep unfinished readings out of production.
* Repeat relevant layer checks with real demo indications.
* #26 collects full-panel acceptance after all preceding instrument steps.

## Geometry and paint regression checks

* Use [project-validate-presentation](../../.agents/skills/project-validate-presentation/SKILL.md) for the validation procedure before changing geometry or adding an instrument. This section records layout constraints and regression lessons.
* Validate all 17 intended elements through the development-only fixture before narrowing attitude space.
* The accepted minimum is 320 CSS px wide and 240 CSS px of usable height after browser bars and safe areas.
* Preserve the sample deadline during resize and redraw through the latest-sample frame scheduler.
* Test painted symbol extents, including stroke clearance, rather than only instrument-region bounds.
* Exclude hidden SVG labels and intersect visible bounds with their ancestor clips when checking content collisions.
* After changing a browser viewport, wait for SVG `viewBox` dimensions to match its CSS bounds before measuring glyphs. A fresh sample alone does not confirm completed resize geometry.
* For #37 and later, distinguish overlapping background coverage from non-overlapping indication content using the [painted-bound checks](../efi/rendering.md#painted-bounds-and-clipping-checks).
* Keep desktop Edge evidence separate from required real iOS Safari and Android Chrome acceptance.

### Paint order checks

Apply these checks when changing SVG layers.

* Check points inside intersecting painted strokes or fills, including transforms and clipping.
* Use visible hit order at those points to check which object covers the other.
* Do not infer painted overlap solely from bounding boxes or render-call order.
* Temporarily reverse the relevant SVG node order to check whether the test detects the wrong layer.
* Apply [test sensitivity and cleanup rules](evidence.md#select-evidence-by-failure-risk) when using temporary mutations.
* Preserve controlled inputs and baseline provenance under the [screenshot comparison procedure](evidence.md#screenshot-comparisons).
* Include capture, pinned edges, both directions, signed values, missing data, expiry, and recovery when applicable.

The [altitude trend acceptance procedure](../efi/altitude-trend.md#acceptance-scenarios) gives the concrete #63 scenarios.

## Feature acceptance ownership

Each instrument feature issue owns:

* Typed data, shared deterministic demo, SSE consumer, and visible indication.
* Focused tests and affected documentation.
* Readability evidence from real iOS Safari and Android Chrome, starting with the first slice.

#26 also owns full-panel acceptance of all 17 elements.
This does not expand #7 or establish live EFI compatibility.

Acceptance criteria:

* [ ] Missing data never becomes zero.
* [ ] An invalid optional instrument value affects only that indication.
* [ ] Source/transport loss invalidates the panel.
* [ ] Reconnect, fresh data after resume, and sample expiry remain correct through every slice.
* [ ] Level, nose up/down, left/right bank, and combined attitudes display correctly.
* [ ] Network/server interruptions recover automatically.
* [ ] Paused, disconnected, invalid, and stale states remain explicit.
* [ ] Unchanged attitude with fresh callbacks remains valid.

> These criteria define expected behavior, not completion status. Record results in the owning issues.
