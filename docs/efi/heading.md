# Current and selected heading

Magnetic heading scale, current readout, and selected-heading bug.

## Requirements

* Use independent source-supplied magnetic headings in `[0, 360)`.
* Display north as `360`, while treating source zero as valid north and raw 360 as invalid.
* Keep ground track outside scope. Do not substitute it for heading.
* Keep a separate numeric selected-heading readout and controls outside scope.

Wire definitions: [telemetry contract](../telemetry-contract.md#optional-magnetic-headings).

## Presentation

### Input availability and display limits

| Input condition | Presentation |
| --- | --- |
| Current heading in `[0, 360)` | Display the scale and current readout |
| Source zero | Valid north, displayed as `360` |
| Raw 360 | Invalid heading |
| Missing or invalid current heading | Red `X`. Hide the scale and bug |
| Missing or invalid selected heading | Hide only the bug |
| Missing altitude, pressure, or other optional fields | Preserve a valid heading |

### Heading scale and current readout

* Use minor ticks every degree, major ticks every 5 degrees, and labels every 10 degrees.
* Display labels with three digits and north as `360`.
* Round the current value to whole degrees and display three digits without a unit suffix.
* Calculate the scale from unrounded heading and wrap continuously across north in either direction.
* Hide whole labels that intersect the readout or either clip edge. Do not clip individual digits.

### Selected-heading bug and north crossing

* Use the shortest angular difference in `[-180, 180)` for selected position. Exactly 180 degrees goes left.
* Pin the bug inside either edge, preserving the whole symbol and its stroke.
* For current 359 degrees and selected 1 degree, place the bug 8 px right of center.
* At capture, retain cyan around the current-heading pointer without covering its text or border.

## Implementation

### Composition and visual object interface

`heading.js` owns its SVG, formatting, geometry, availability, and pure calculations.
`createHeading(instrument, layers)` exposes these operations:

| Operation | Contract |
| --- | --- |
| `resize(layout)` | Use `layout.heading` to update geometry and grow the retained tick pool only when needed |
| `render(frame)` | Consume the complete frame and update current and selected-heading presentation |
| `invalidate()` | Clear availability, scale visibility, and bug position. Show `X` |

The shared panel can pass an invalidation reason, but heading does not use that argument.

### Calculations and formatting

| Function | Responsibility |
| --- | --- |
| `validHeading` | Accept finite values in `[0, 360)` |
| `headingText` | Round to whole degrees, format three digits, and display north as `360` |
| `headingDifference` | Calculate the shortest angular difference, with the 180-degree tie going left |
| `headingBugPosition` | Position the selected bug and clamp its center with an 8.5 px inset |
| `headingTicks` | Calculate unrounded scale positions, wrapped values, major ticks, and whole-label visibility |

### Geometry, clipping, and paint order

| Parameter | Requirement |
| --- | --- |
| Allocation | `layout.heading` in the existing 32 CSS px strip between IAS and altitude |
| Layer and clip | Layer 3 (`symbols`), with its own exclusive screen-space heading tape clip |
| Scale | Fixed 4 CSS px per degree |
| Labels | Monospace 11 px |
| Current value | Monospace 14 px |
| Window | Centered 44 by 22 px rectangle, with a lower pointer 16 px wide and 8 px high |
| Colors | Existing text and value-window tokens, with a white 1 px border |
| Selected bug | Cyan 16 by 7 px block at the tape's bottom, with a complementary upper notch |
| Bug edge inset | 8.5 px for the center, preserving the whole symbol and stroke |

* Share pointer and notch dimensions in one module.
* Leave horizon geometry unchanged.

Paint heading elements in this order:

1. Background.
2. Scale.
3. Selected bug.
4. Opaque black current window and its border.
5. Current value.

### Frame updates, SVG state, and accessibility

* Resize updates geometry and grows the retained tick pool only when needed.
* Use shared frame scheduling and the original expiry deadline without history, filtering, interpolation, or a new timer.
* Include raw magnetic heading and valid selected heading in the accessible label. Identify unavailable selected heading separately.

| State or DOM property | Effect or use |
| --- | --- |
| Heading `data-state` | Record current-heading availability. CSS makes the unavailable value red |
| Heading `data-selected-available` | Record selected-heading availability. The attribute does not itself control visibility |
| Scale and bug `style.display` | Hide the scale and bug on invalidation. Hide only the bug when selected heading is unavailable |
| Label `style.visibility` | Hide whole labels that intersect the window or clip edges |
| Bug `transform` | Position an available bug. Remove its position when unavailable |
| Heading `aria-label` | Describe raw headings or the unavailable state |

## Testing

Use the [validation scope matrix](../testing/index.md#validation-by-change-scope) and the feature checks below.
Shared browser commands, phone setup, and evidence rules are linked from those checks.
Record the tested commit, Windows/MSVC and browser versions, command results, and screenshot paths in #26 or its PR.

### Acceptance scenarios

Use the [magnetic-heading contract](../telemetry-contract.md#optional-magnetic-headings) and [presentation rules](#presentation).

| Scenario | Expected result |
| --- | --- |
| Source zero and raw 360 | Zero displays as `360`. Raw 360 is invalid |
| North crossing in either direction | Continuous unrounded scale movement and correct rounded readout |
| Current 359 degrees, selected 1 degree | Bug exactly 8 CSS px right of center |
| Exactly 180 degrees difference | Bug goes left |
| Capture | Cyan remains around the black pointer without covering text or border |
| Either pinned edge | Whole bug and stroke remain visible |
| Local input loss | Current-heading loss hides scale and bug. Selected-heading loss hides only the bug |

### Automated checks and independent fixtures

Run the existing [automated Rust and Node commands](../testing/automated.md#automated-checks) before the desktop browser scripts.

| Check | Required evidence |
| --- | --- |
| Rust/SSE | Canonical `[0, 360)` values, valid zero, independent absence, shared sequence/age/expiry, clearing on state publications, and identical samples for two clients |
| Scenario | Independent phase checkpoints, both north crossings, +/-3 deg/s agreement, 360-sample heading repeat and unchanged 3960-sample altitude cycle |
| Node | Three digits, degree formatting, rounding near north, shortest angular distance, 180-degree left tie, whole bug edge pinning, and whole-label hiding |
| Lifecycle | Latest shared sample, expiry before drawing, resize without refreshed age, pause, disconnect, reconnect and background/resume without another subscription |

### Desktop geometry and availability transitions

Run both desktop browser scripts in the [shared EFI checks](../testing/pfd.md#responsive-efi-layout-and-desktop-checks).
The fixture uses the production heading module, without ground-track or separate numeric selected-heading illustrations.

* Check real glyph and stroke bounds, fixed 4 CSS px/degree spacing, cyan notch capture, and both edges.
* Check stable nodes and independent unavailable states.
* Validate painted visibility rather than bounds alone. Hidden heading labels can retain nonzero bounds outside their painted clip.

Inspect the browser evidence:

1. Inspect `.local/pfd-browser/heading-production-*` and `heading-layout-*` for north, capture, and both pinned edges.
2. Confirm the current 359-degree / selected 1-degree bug is exactly 8 CSS px right of center.
3. Confirm cyan remains around the black pointer at capture, without covering its text or border.

### Controlled heading inputs

The fixture accepts these independent heading inputs:

```json
{"heading_deg":359,"selected_heading_deg":1}
{"heading_deg":1,"selected_heading_deg":359}
{"heading_deg":0,"selected_heading_deg":0}
{"heading_deg":354,"selected_heading_deg":90}
{"heading_deg":354,"selected_heading_deg":180}
{"heading_deg":354,"selected_heading_deg":null}
{"heading_deg":null,"selected_heading_deg":1}
{"heading_deg":360,"selected_heading_deg":1}
```

### Real-phone checks

Apply the [shared phone procedure](../testing/devices.md#real-phone-acceptance-procedure) for #26.
Include full-panel results and the [controlled heading scenarios](#controlled-heading-inputs).

> Keep real-phone evidence separate from desktop checks. Desktop results do not establish actual iOS Safari or Android Chrome acceptance.

### Full-panel integration

* Check all 17 included illustration items, altitude trend, target alerts, and both orientations from 320 by 240 CSS px.
* Check heading wrap at 359/0 during full-panel acceptance.
* Inspect `.local/pfd-runtime-heading-north.png` for the compiled Rust demo's north crossing.

The current `tests/browser/helpers/layout-fixture.js` declares 17 illustration IDs in `marker.dataset.items`:
`1,2,3,4,5,7,8,9,10,11,12,13,15,16,18,20,21`.
The fixture server serves the production page with the development fixture module added for `/layout.html`.
This count identifies illustration items, not the number of SVG nodes or proof of visual acceptance.

To check the declared item count from the repository root:

```powershell
$fixtureText = Get-Content tests/browser/helpers/layout-fixture.js -Raw
$fixtureItems = [regex]::Match($fixtureText, "marker\.dataset\.items = '([^']+)'").Groups[1].Value -split ','
$fixtureItems.Count
```

The expected result is `17`.

## Rationale and regression lessons

Garmin G5 Guide 190-01112-12 Rev. 2, section 3.3.8, provides the presentation reference.

> Fixed spacing, edge pinning, dimensions, and independent availability are application design decisions.

Preserve these validation lessons and decision boundaries:

* North displays as `360`, but raw 360 is invalid. Distinguish source validation from display formatting.
* The 359/1-degree capture case and 180-degree left tie detect wrap and direction errors.
* Check actual cyan paint at capture and whole-label hiding at both the current window and clip edges.
* The heading scope excludes ground-track fallback and a separate numeric selected heading.
* Use the full 17-element fixture for accumulated acceptance. Keep real-phone evidence separate from desktop checks.
