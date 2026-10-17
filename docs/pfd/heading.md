# Current and selected heading

Magnetic heading scale, current readout, and selected-heading bug.

## Requirements

Use independent source-supplied magnetic headings in `[0, 360)`. Zero is valid north; raw 360 is invalid.
Display north as `360`. Ground track, a separate numeric selected-heading readout, and controls remain outside scope.

Wire definitions: [telemetry contract](../telemetry-contract.md#optional-magnetic-headings).

## Presentation

The scale and current readout use these dimensions:

| Element | Rule |
| --- | --- |
| Scale | Fixed 4 CSS px per degree. Minor ticks every degree, major ticks every 5 degrees |
| Labels | Every 10 degrees, three digits, monospace 11 px. North is `360` |
| Current value | Three digits without a unit suffix, rounded to whole degrees, monospace 14 px. North is `360` |
| Window | Centered 44 by 22 px rectangle, with a lower pointer 16 px wide and 8 px high |
| Colors | Existing text and value-window tokens, with a white 1 px border |
| Selected bug | Cyan 16 by 7 px block at the tape's bottom, with a complementary upper notch |

Geometry and paint order preserve capture and edge clearance:

* The pointer and notch share dimensions in one module.
* The scale paints first, then the bug, then the opaque black window, border, and current value.
* At capture, cyan remains visible around the pointer.
* Selected position uses the shortest angular difference in `[-180, 180)`. Exactly 180 degrees goes left.
* Clamp the bug center with an 8.5 px inset, preserving the whole symbol and its stroke at either edge.
* For current 359 degrees and selected 1 degree, the bug is 8 px right of center.
* Scale calculations use unrounded heading and wrap continuously across north in either direction.
* Hide entire labels that intersect the readout or either clip edge. Do not clip individual digits.

Availability remains local:

* Missing or invalid current heading shows red `X`, hiding the scale and bug.
* Missing or invalid selected heading hides only the bug.
* Missing altitude, pressure, or other optional fields does not affect a valid heading.
* Ground track does not substitute for heading. No separate selected-heading readout or control is provided.

Garmin G5 Guide 190-01112-12 Rev. 2, section 3.3.8, provides the presentation reference.
Fixed spacing, edge pinning, dimensions, and independent availability are application design decisions.

## Implementation

`heading.js` owns its SVG, formatting, geometry, availability, and pure calculations:

* The module occupies `layout.heading` in the existing 32 CSS px strip between IAS and altitude.
* It uses layer 3 (`symbols`) with its own screen-space clip. Horizon geometry remains unchanged.
* Resize updates geometry and grows the retained tick pool only when needed.
* Render consumes the complete frame. Invalidation clears availability, scale visibility, and bug position.
* Shared frame scheduling and the original expiry deadline apply without history, filtering, interpolation, or a new timer.

### Factory interface

| Factory | Returned operations |
| --- | --- |
| `createHeading(instrument, layers)` in `heading.js` | `resize(layout)`, `render(frame)`, `invalidate(reason)` |

### SVG paint order

| Scope | Order |
| --- | --- |
| Heading | Background, scale, selected bug, current window, current value |
| Heading definitions | Exclusive heading tape clip |

### SVG and CSS state

| State written by JavaScript | CSS or presentation effect |
| --- | --- |
| Heading `data-state`, `data-selected-available` | Local red `X`, scale visibility and independent selected-bug visibility |

## Testing

Use the [validation scope matrix](../testing/index.md#validation-by-change-scope) and the feature checks below.
Shared browser commands, phone setup, and evidence rules are linked from those checks.

### Heading acceptance

Use the [magnetic-heading contract](../telemetry-contract.md#optional-magnetic-headings) and [presentation rules](#presentation).
Run the existing Rust and Node commands, then both desktop browser scripts in the [shared PFD checks](../testing/pfd.md#responsive-pfd-layout-and-desktop-checks).

| Check | Required evidence |
| --- | --- |
| Rust/SSE | Canonical `[0, 360)` values, valid zero, independent absence, shared sequence/age/expiry, clearing on state publications, and identical samples for two clients |
| Scenario | Independent phase checkpoints, both north crossings, +/-3 deg/s agreement, 360-sample heading repeat and unchanged 3960-sample altitude cycle |
| Node | Three digits, degree formatting, rounding near north, shortest angular distance, 180-degree left tie, whole bug edge pinning, and whole-label hiding |
| SVG | Real glyph and stroke bounds, fixed 4 CSS px/degree spacing, cyan notch capture, both edges, stable nodes and independent unavailable states |
| Lifecycle | Latest shared sample, expiry before drawing, resize without refreshed age, pause, disconnect, reconnect and background/resume without another subscription |
| Full panel | All 17 included elements, altitude trend, target alerts, and both orientations from 320 by 240 CSS px |

Inspect the browser evidence:

1. Inspect `.local/pfd-browser/heading-production-*` and `heading-layout-*` for north, capture, and both pinned edges.
2. Confirm the current 359-degree / selected 1-degree bug is exactly 8 CSS px right of center.
3. Confirm cyan remains around the black pointer at capture, without covering its text or border.
4. Inspect `.local/pfd-runtime-heading-north.png` for the compiled Rust demo's north crossing.
5. Record the tested commit, Windows/MSVC and browser versions, command results, and screenshot paths in #26 or its PR.

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

Apply the [shared phone procedure](../testing/devices.md#real-phone-acceptance-procedure) for #26, including full-panel results and the controlled heading scenarios above.

### Desktop fixture scope

| Check | Expected result |
| --- | --- |
| Heading strip | Production heading module, without ground-track or separate numeric selected-heading illustrations |

### Full-panel geometry

Check heading wrap at 359/0 during full-panel acceptance.

### Clipping and scope provenance

Source: #66 browser check correction, 2026-10-11; retained hidden heading labels had nonzero bounds outside their painted clip.
Source: accepted #26 scope amendment, 2026-10-11. Scope excludes ground-track item 19. The full fixture contains 17 elements and uses production heading SVG.

## Rationale and regression lessons

Preserve these validation lessons and decision boundaries:

* North displays as `360`, but raw 360 is invalid. Distinguish source validation from display formatting.
* The 359/1-degree capture case and 180-degree left tie detect wrap and direction errors.
* Check actual cyan paint at capture and whole-label hiding at both the current window and clip edges.
* The accepted #26 scope excludes ground-track fallback and a separate numeric selected heading.
* Use the full 17-element fixture for accumulated acceptance. Keep real-phone evidence separate from desktop checks.

Source: accepted #26 plan and Garmin section 3.3.8, 2026-10-11.

Source: accepted #65 correction, 2026-10-11, removing the degree symbol from the current heading window.

Source: #63 reclaimed four pixels for the central allocation, as reflected in `web/layout.js`, reviewed 2026-10-11.
