# True airspeed

The independent TAS readout above the IAS tape.

## Requirements

* Display source-supplied TAS independently of IAS, GS, profile, and acceleration.
* Apply the 999 kt presentation limit before rounding.
* Use the 32 CSS px top-left allocation without moving heading or reducing the central attitude area.

Wire definitions: [telemetry contract](../telemetry-contract.md#optional-speed-indications).

## Presentation

### Input availability and display limits

Classify TAS before rounding its displayed value:

| Source value | Indication |
| --- | --- |
| Finite nonnegative value through 999 kt | Round to the nearest knot, with halves rounded up and no leading zeros |
| Value above 999 kt | Red `OVR` |
| Missing or invalid value | Red `X` |

* TAS does not depend on IAS, GS, profile, or acceleration.
* The label stays white and visible during local failure.

### Readout and unit labels

* The independent TAS readout occupies the top-left area above the IAS tape.
* The label `TAS` is left-aligned and the value is right-aligned.
* The visible value has no unit suffix. The persistent `[KT]` tape label and accessible description identify knots.

## Implementation

### Composition and visual object interface

| Module or helper | Responsibility |
| --- | --- |
| `airspeed/true-airspeed.js` | Own TAS state, rounding, top-strip geometry, SVG, and local availability |
| `speedState` | Classify speeds using the [shared speed helpers](airspeed.md#shared-speed-helpers) |
| `airspeed/index.js` | Pass the top-left allocation and current TAS value through the [airspeed composition](airspeed.md#composition-and-visual-object-interfaces) |

| Object / factory | Owned content | Resize input | Render input |
| --- | --- | --- | --- |
| TAS: `createTrueAirspeed(parent)` | Independent TAS group/background/frame/label/value and local availability | `{ x, y, width, height }` | TAS value |

### Geometry and styling

| Element | Geometry or styling contract |
| --- | --- |
| TAS field | Start at SVG `(0, 0)`, share the tape width, and end at the tape's top edge at 32 CSS px |
| Background | Opaque `--value-window` |
| Frame | White `--text`, with a 1 CSS px stroke inset by 0.5 CSS px |
| Label and value | 3 CSS px side padding and a common vertical center |
| Typography | GS font family, 11 CSS px, weight 600, and white `--text` for both texts |
| Failure and overflow marks | Red `X` and `OVR` retain the same 11 CSS px size |

* TAS does not move heading, alter tape clips, or reduce central attitude space.

### Frame updates and accessibility

* Each render replaces the previous reading.
* TAS shares the frame deadline and full-panel unavailable cover without its own cache or timer.
* The accessible description identifies knots.

## Testing

Use the [validation scope matrix](../testing/index.md#validation-by-change-scope) and the feature checks below.
Shared browser commands, phone setup, and evidence rules are linked from those checks.

The checks on this page define TAS acceptance, including separate real-device results.

### Acceptance scenarios

Use the [speed contract](../telemetry-contract.md#optional-speed-indications) and [TAS presentation](#presentation).

| Check | Expected behavior |
| --- | --- |
| Browser | Source-supplied TAS reaches SVG and ARIA. Explicit expectations cover zero, fractions, rounding, 999, overflow, missing/null/malformed input, current attitude-only publications, expiry and recovery |
| Partial loss | Missing TAS preserves valid IAS/GS. Missing IAS or GS preserves TAS |
| Geometry | Stable production nodes in both compositions. Glyph containment for one-, two-, three-digit values, X and OVR. Inset frame, opaque background, and clearance from heading and tape |

### Automated checks and desktop geometry

Run the [automated checks](../testing/automated.md#automated-checks) and both [desktop scripts](../testing/pfd.md#responsive-efi-layout-and-desktop-checks).

| Check artifact | Coverage or inspection |
| --- | --- |
| `tests/browser/checks/true-airspeed-browser-checks.mjs` | 320×480, 326×246, and 568×240 at whole/fractional bounds and simulated safe-area padding |
| `.local/pfd-browser/tas-*` screenshots | Inspect over sky and ground |

### Real-phone checks

1. Apply the [shared phone procedure](../testing/devices.md#real-phone-acceptance-procedure).
2. Use [phone fixtures](../testing/devices.md#phone-fixture-setup) in both compositions. Enter these controlled samples:

   ```json
   {"ias_kt":150,"tas_kt":160,"gs_kt":170}
   {"tas_kt":0}
   {"tas_kt":85.5}
   {"tas_kt":999}
   {"tas_kt":999.01}
   {"tas_kt":null}
   {"tas_kt":"85"}
   {"tas_kt":160,"omit_tas":true}
   {"omit_tas":false,"tas_kt":160,"ias_kt":null}
   {"ias_kt":150,"gs_kt":null}
   {"gs_kt":170,"hold":true}
   {"hold":false}
   ```

3. Check continuous backgrounds and readable TAS labels throughout the shared rotation, layout, and recovery scenarios.

> Actual iOS Safari and Android Chrome acceptance remains separate from desktop checks.

### Full-panel integration

| Check | Expected behavior |
| --- | --- |
| Rust and SSE | Independently optional finite nonnegative TAS, the v1 envelope, atomic publication, shared age, two matching clients, and clearing on omission or source loss |
| Demo | Independent speed checkpoints repeat, including simultaneous IAS 150, TAS 160, and GS 170 |
| Runtime | Embedded TAS module supports HTTP/MIME and HEAD checks. Real demo readouts recover after server restart without reload |

## Rationale and regression lessons

* Use the top-left 32 CSS px allocation without moving heading or narrowing attitude.
* Keep the TAS frame white. GS retains its speed-trend magenta frame.
* TAS uses source-supplied data independently of IAS, GS, configuration, and acceleration.
* The 999 kt presentation limit applies before rounding. 999.01 kt is valid telemetry with an overflow indication.
* Exercise browser omission and malformed input after a valid TAS reading so an existing X cannot satisfy the transition check.
* Reconnect can finish after the demo's initial zero-speed segment. Verify the recovered sample instead of assuming zero.
