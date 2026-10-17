# True airspeed

The independent TAS readout above the IAS tape.

## Requirements

Display source-supplied TAS independently of IAS, GS, profile, and acceleration. Apply the 999 kt presentation limit before rounding.
Use the existing 32 CSS px top-left allocation without moving heading or reducing the central attitude area.

Wire definitions: [telemetry contract](../telemetry-contract.md#optional-speed-indications).

## Presentation

The independent TAS object occupies the existing top-left area above the IAS tape:

* The field starts at SVG `(0, 0)`, shares the tape width, and ends at its top edge at 32 CSS px.
* Background fill uses opaque `--value-window`. A white `--text` frame has a 1 CSS px stroke inset by 0.5 CSS px.
* The label `TAS` is left-aligned and the value is right-aligned with 3 CSS px side padding and a common vertical center.
* Both texts use the GS font family, 11 CSS px, weight 600, and white `--text`.
* Finite nonnegative source values through 999 kt round to the nearest knot, with halves rounded up and no leading zeros.
* The visible value has no unit suffix. The persistent `[KT]` tape label and accessible description identify knots.
* Missing or invalid values show red `X`; values above the display limit show red `OVR`, using the same 11 CSS px size.
* The label stays white and visible during local failure. TAS does not depend on IAS, GS, profile, or acceleration.
* Each render replaces the previous reading. TAS shares the frame deadline and full-panel unavailable cover without its own cache or timer.
* TAS does not move heading, alter tape clips, or reduce central attitude space.

Real-device acceptance belongs to [#66](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/66).

## Implementation

`airspeed/true-airspeed.js` owns TAS state, rounding, top-strip geometry, SVG, and local availability.
It uses `speedState` from the [shared speed helpers](airspeed.md#shared-speed-helpers) for speed classification.
`airspeed/index.js` passes the top-left allocation and the current TAS value through the [airspeed composition](airspeed.md#implementation).
Each render replaces the reading and uses the common frame deadline.

### Visual object interface

| Object / factory | Owned content | Resize input | Render input |
| --- | --- | --- | --- |
| TAS: `createTrueAirspeed(parent)` | Independent TAS group/background/frame/label/value and local availability | `{ x, y, width, height }` | TAS value |

## Testing

Use the [validation scope matrix](../testing/index.md#validation-by-change-scope) and the feature checks below.
Shared browser commands, phone setup, and evidence rules are linked from those checks.

### True airspeed acceptance

Issue [#66](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/66) owns TAS acceptance.
Use the [speed contract](../telemetry-contract.md#optional-speed-indications) and [TAS presentation](#presentation).

| Check | Expected behavior |
| --- | --- |
| Rust and SSE | Independently optional finite nonnegative TAS, unchanged v1 envelope, atomic publication, shared age, two matching clients, and clearing on omission or source loss |
| Demo | Independent speed checkpoints repeat, including simultaneous IAS 150, TAS 160, and GS 170 |
| Browser | Source-supplied TAS reaches SVG and ARIA; zero, fractions, rounding, 999, overflow, missing/null/malformed input, old snapshots, expiry and recovery have explicit expectations |
| Partial loss | Missing TAS preserves valid IAS/GS. Missing IAS or GS preserves TAS |
| Geometry | Stable production nodes in both compositions; glyph containment for one-, two-, three-digit values, X and OVR; inset frame, opaque background, and clearance from heading and tape |
| Runtime | Embedded TAS module supports HTTP/MIME and HEAD checks; real demo readouts recover after server restart without reload |

Run the [automated checks](../testing/automated.md#automated-checks) and both [desktop scripts](../testing/pfd.md#responsive-efi-layout-and-desktop-checks).
`tests/browser/checks/true-airspeed-browser-checks.mjs` covers 320×480, 326×246, and 568×240 at whole/fractional bounds and simulated safe-area padding.
Inspect `.local/pfd-browser/tas-*` screenshots over sky and ground.

Apply the [shared phone procedure](../testing/devices.md#real-phone-acceptance-procedure) for #66, using [phone fixtures](../testing/devices.md#phone-fixture-setup) in both compositions.
Enter these controlled samples:

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

Check continuous backgrounds and readable TAS labels throughout the shared rotation, layout, and recovery scenarios.

## Rationale and regression lessons

* Use the existing top-left 32 CSS px allocation without moving heading or narrowing attitude.
* Keep the TAS frame white; GS retains its speed-trend magenta frame.
* TAS uses source-supplied data independently of IAS, GS, configuration, and acceleration.
* The 999 kt presentation limit applies before rounding; 999.01 kt is valid telemetry with an overflow indication.
* Exercise browser omission and malformed input after a valid TAS reading so an existing X cannot satisfy the transition check.
* Reconnect can finish after the demo's initial zero-speed segment; verify the recovered sample instead of assuming zero.

Source: accepted #66 implementation plan and formatting choices, 2026-10-11.

Source: release-runtime restart check on 2026-10-11 demonstrated a nonzero fresh TAS after reconnect.

Evidence limit: actual iOS Safari and Android Chrome acceptance remains separate from desktop checks.
