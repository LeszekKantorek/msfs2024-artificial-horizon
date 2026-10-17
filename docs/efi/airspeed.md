# Indicated airspeed and ground speed

IAS tape, rolling digits, independent GS, and the shared airspeed composition.

## Requirements

Display source-supplied IAS and GS independently in knots. Zero is valid; missing data must not become zero or retain a previous reading.
The 0–999 kt range limits presentation, not valid telemetry. Profile and acceleration availability remain independent of the speed readings.

Wire definitions: [telemetry contract](../telemetry-contract.md#optional-speed-indications).

## Presentation

Composition and geometry use the panel's airspeed allocation:

* IAS, TAS, and GS use layer 1.
* IAS and GS local failure and overflow marks use layer 4. TAS marks stay inside its opaque layer-1 field.
* The airspeed rectangle defines tape geometry and includes the 1 CSS px increase for wider range bands and scale-label clearance.
* The central attitude region starts 1 CSS px farther right and is 1 CSS px narrower. Altitude retains its allocation.
* GS stays at the bottom of the panel, with the same left edge and width as IAS.
* The IAS background extends to GS without a gap.
* IAS and altitude scales use clips of equal height over their complete existing allocations.
* `[KT]` and `[FT]` are centered in their tape widths at panel height minus 40 CSS px, with the same 10 CSS px font and baseline.
* The labels occupy the 16 CSS px band between each scale and footer, with 1 CSS px above and below their desktop glyph bounds.
* `[KT]` remains visible during IAS loss; the fallback reference list has no additional unit heading.
* GS uses the same opaque dark background and inset 1 CSS px frame as BARO, with the speed-trend magenta token `--turn` for its border.
* GS and BARO occupy aligned 32 CSS px footers. Both tape backgrounds meet their footer without a gap and both footers meet the bottom panel edge.
* The exterior three-pixel margins and safe-area handling remain unchanged.

The tape and readouts use these display rules:

* Tape spacing is 4 CSS px/kt, with major labels every 10 kt and minor ticks every 2 kt.
* The scale and rolling digits use fractional received IAS without rounding the input to whole knots.
* Screen-space clipping hides readings outside the tape rectangle and its unit label.
* The fixed opaque window clips three rolling digit columns.
* Leading zeros are blank.
* Units roll from fractional IAS.
* Higher columns roll during the final knot before a decimal carry.
* GS uses a separate labelled readout, rounded to the nearest knot with halves rounded up.
* GS has no visible unit suffix; its accessible label retains knots.
* GS label and value share 13 CSS px, weight-600 text, with a common vertical center and three-pixel left/right padding.

Availability and lifecycle remain independent:

* Display values span 0–999 kt.
* Above 999 kt, show `OVR`.
* Unavailable values show `X`.
* Neither missing nor overflowing IAS leaves a visible numeric tape.
* TAS and GS availability remain independent.
* No independent animation, timer, interpolation, or client-generated flight values are used.

## Implementation

`airspeed/index.js` composes IAS, TAS, and GS once, exposing `resize(layout)` and `render(frame)`:

* Compute the shared GS panel position once per resize.
* Pass `layout.airspeed` and the shared position to IAS.
* Pass `{ x, y, width }` to GS and the top-left `{ x, y, width, height }` allocation to TAS.
* Select IAS/profile/acceleration and TAS/GS from the complete frame during render.
* Keep coherent IAS readings and child geometry in IAS.
* Keep GS state, rounding, and geometry in GS.

The following objects implement IAS and GS. [Ranges/references](airspeed-indications.md#visual-object-interfaces) and [TAS](true-airspeed.md#visual-object-interface) document their own interfaces.

| Object / factory | Owned content | Resize input | Render input |
| --- | --- | --- | --- |
| IAS: `createIas(defs, parent, warnings)` | IAS root/background, shared tape host and tape clip, state/ARIA flags, child composition | IAS allocation and shared GS panel Y | IAS value, profile, acceleration |
| Readout: `createReadout(defs, iasHost, warnings)` | Exclusive digit clip, window, columns, unit, IAS warning | Readout geometry and unit position | IAS reading and fallback-reference flag |
| GS: `createGroundSpeed(parent, warnings)` | Independent GS group/background/label/value and warning | `{ x, y, width }` | GS value |

Ownership and availability remain local:

* IAS owns the clip shared by tape and reference markers.
* Readout owns its exclusive digit clip.
* IAS, TAS, GS, profile, and acceleration availability remain independent.
* Every frame resets relevant warning, visibility, unit, fallback, and trend states, including missing data.

### Local calculations and constants

The speed objects keep calculations with their visual implementations:

| Module | Calculations and constants |
| --- | --- |
| `airspeed/ias.js` | IAS readings, `speedIndications`, and child geometry |
| `airspeed/ground-speed.js` | GS state, rounding, and geometry |
| `airspeed/readout.js` | `rollingDigits` and local `DIGIT_HEIGHT` of 22 px |

IAS child geometry and GS geometry stay inside their owning resize operations.
The [range and reference objects](airspeed-indications.md#implementation) own their calculations and tape constants.
[TAS](true-airspeed.md#implementation) owns its independent reading and top-strip geometry.

### Shared speed helpers

`airspeed/helpers.js` exports only `MAX_SPEED`, `speedState`, and `tapeOffset` for actual shared rules:

| Rule | Consumers / value |
| --- | --- |
| `speedState` | IAS, TAS, and GS |
| `tapeOffset` | Tape and references, using 4 px/kt |
| `MAX_SPEED` | Speed classification, tape, and references |

### Validation and display limits

* Validation accepts finite nonnegative speeds above the display limit.
* The 999 kt limit, `OVR`, and six-second prediction belong to airspeed calculations.

### Factory interface

| Factory | Returned operations |
| --- | --- |
| `createAirspeed(instrument, layers)` in `airspeed/index.js` | `resize(layout)`, `render(frame)` |

### Object composition

```text
Airspeed
|-- IAS
|   |-- Tape (scale, ranges, trend)
|   |-- Readout (window, digits, unit, warning)
|   `-- References (markers, leaders, fallback values)
|-- True airspeed
`-- Ground speed
```

### Layers and local clips

| Element | Layer / origin | Clip and owner |
| --- | --- | --- |
| IAS scale/ranges/trend/reference markers | Layer 1, IAS origin | IAS shared tape clip |
| IAS rolling digits | Layer 1, IAS origin | Readout exclusive digit clip |

### SVG paint order

| Scope | Order |
| --- | --- |
| IAS tape | Scale, ranges, trend, references |
| IAS | Background, tape, window, digits, unit, fallback references |
| Airspeed layer | IAS, GS, TAS |

### SVG and CSS state

| State written by JavaScript | CSS or presentation effect |
| --- | --- |
| IAS `data-state` | Hide tape and digits unless the value is available |
| IAS `data-overspeed`, `data-approaching` | Red pointer window and yellow predictive-warning digits |

### Frame state and accessibility

* IAS root state updates run before its child indication updates on every frame.
* TAS and GS update their states independently.
* IAS, TAS, and GS reset warning colors, availability, local X/OVR marks, profile visibility, and fallback flags.
* Accessible speed labels retain current values or unavailable/overflow states.

## Testing

Use the [validation scope matrix](../testing/index.md#validation-by-change-scope) and the feature checks below.
Shared browser commands, phone setup, and evidence rules are linked from those checks.

For changes to footer geometry, also apply the [shared GS/BARO and unit-label checks](altitude.md#tape-footers-and-source-formatted-baro).

### Airspeed and ground-speed acceptance

Run the automated checks and both desktop browser scripts for airspeed implementation changes.
Check these speed-specific behaviors:

| Area | Expected behavior |
| --- | --- |
| Compatibility | Old snapshots show attitude with unavailable IAS/GS. Unsupported envelopes remain globally invalid |
| Input | Zero and fractional speeds are valid. Negative/non-finite/malformed/missing speeds affect only their own indication |
| Presentation | IAS major labels every 10 kt, minor ticks every 2 kt, fixed pointer, rolling digits, no negative scale labels |
| Rollover | Check 9.5 → 10, 99.5 → 100, and reverse transitions. Leading zeros stay blank |
| Display limits | 999 kt is readable. 999.01 and 1000 kt show `OVR`. Missing or invalid values show `X`, never zero |
| GS | 137.6 kt shows 138 beside the GS label, without a visible unit suffix. The accessible description retains knots |
| Lifecycle | Equal fresh speeds renew validity. Resize preserves the accepted deadline. Reconnect/resume require a new fresh sample without duplicate subscriptions |
| Integration | Two SSE clients receive matching IAS/GS. The compiled Rust demo supplies 150/170 kt during its hold segment |
| Fixture | Items 1/4/8 use the real speed module, without duplicate speed readouts. All other included items also use production modules |

#### Real-phone checks

Apply the [shared phone procedure](../testing/devices.md#real-phone-acceptance-procedure) for #21.
Check IAS acceleration, hold, deceleration, and distinct GS, including digits, ticks, and local failure marks against sky and ground.
Use [phone fixtures](../testing/devices.md#phone-fixture-setup) for controlled rollover, limits, and failures with these inputs:

```json
{"ias_kt":99.5,"gs_kt":137.6}
{"ias_kt":100,"gs_kt":999}
{"ias_kt":999.01,"gs_kt":1000}
{"ias_kt":null,"gs_kt":137}
{"ias_kt":123,"gs_kt":null}
{"ias_kt":123,"gs_kt":137,"hold":true}
{"hold":false}
```

### Focused calculation and lifecycle suites

| Suite | Coverage / method |
| --- | --- |
| Speed calculations | `tests/airspeed.test.mjs` and `tests/airspeed-ranges.test.mjs` import readout, tape, references, IAS, and shared speed helpers directly |

### Independent fixtures

| Fixture / suite | Coverage |
| --- | --- |
| `tests/fixtures/speed-samples.json` | Independent IAS/GS checkpoints during acceleration, hold, deceleration, and repeat |

### Demo behavior

| Property | Expected behavior |
| --- | --- |
| IAS/GS | 0–2 s: zero. 2–8 s: acceleration to IAS 150 / GS 170 kt. 8–10 s: hold. 10–16 s: deceleration to zero. 16–18 s: zero |

### Composition recovery regression

| Owner | Checks |
| --- | --- |
| #56: recovery | Repeat IAS/profile loss during resize and both recovery orders, with independently painted GS and reset warnings/references |

### Retired asset routes

| Retired route | Current owner |
| --- | --- |
| `/airspeed.js` | `/airspeed/index.js` and its visual objects |
| `/airspeed/calculations.js` | Pure functions in the implementing visual objects |

### Full-panel geometry

Check IAS rolling digits and speed boundaries during full-panel acceptance.

## Rationale and regression lessons

Keep these pitfalls in scope when extending the instrument:

* The accepted 0–999 kt range is a presentation limit, not a telemetry limit or an aircraft speed threshold.
* Exercise the real IAS/GS module in the full fixture, including fallback references that occupy the development badge area.
* Measure fallback reference glyphs against both the IAS warning and persistent `[KT]` when changing footer positions.
* Keep unit-label margins primary when choosing font size, and measure their clearance from the tape clip and footer.
* Reuse the speed-trend magenta token for the GS border and the opaque BARO background for both footers.

Source: accepted implementation plan for [#21](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/21), 2026-10-10, including the explicit 0–999 kt choice.

Source: accepted layout clarification, 2026-10-10, to keep GS at the bottom and replace the IAS/GS background gap with empty tape background.

Source: accepted #65 plan and Edge glyph measurements, 2026-10-11, preserving 16 px fallback row spacing while clearing `[KT]`.

Source: accepted #65 correction, 2026-10-11; Edge 155 measures 1 CSS px above and below both labels at 10 CSS px centered at panel height minus 40 CSS px.
