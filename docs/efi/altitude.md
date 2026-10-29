# Altitude and barometric setting

Signed altitude tape and rolling drums, independent BARO, and the altitude composition.

## Requirements

Display source-supplied altitude for its pressure setting without browser pressure correction.
Altitude and BARO are independently optional. Legacy BARO strings remain verbatim when numeric BARO data is absent.
Finite altitude outside the display range remains valid telemetry.

Wire definitions: [telemetry contract](../telemetry-contract.md#optional-altitude-and-pressure-setting).

## Presentation

The tape and readouts use these presentation rules:

* The tape remains centered on the current altitude, with 20-ft minor ticks and 100-ft major ticks and labels.
* Minor ticks are always 8 CSS px apart and major ticks are 40 CSS px apart.
* This matches the former 400-ft scale at a 160 CSS px tape height in minimum-height landscape.
* Taller tapes show a wider range and more labels without stretching the spacing or text.
* A 160 CSS px tape spans 400 ft and a 400 CSS px tape spans 1000 ft.
* Higher values appear above the pointer.
* The tape includes negative values and crosses zero without a direction change.
* The readout supports -9999 through 99999 ft; valid values beyond that range show `OVR` and hide the scale and digits.
* The lower two digits roll together through 00/20/40/60/80, using the full fractional magnitude.
* Higher columns carry during the final 20 ft before their decimal boundary, with leading zeros hidden.
* Negative values have a separate minus sign and magnitude drums; zero, including negative zero, has no minus.
* An adjacent drum row outside the display range is blank instead of wrapping to zero.
* Missing altitude shows `X` and hides the scale and digits without hiding valid pressure or other instruments.
* BARO occupies the bottom 32 CSS px of the right column, reaching the panel edge.
* A one-pixel cyan border is inset by half a pixel over an opaque value-window background.
* One centered, 11 CSS px, weight-600 cyan text node displays BARO.
* Numeric pressure displays two decimal places in inHg or a whole hPa. Simulator standard mode displays `STD`.
* A touch or click on the field switches the browser-local unit preference. The choice persists across reloads and does not change the simulator.
* When numeric data is absent, legacy strings remain verbatim, including empty strings. Legacy strings cannot be converted.
* Invalid numeric BARO shows a red `X`; a numeric setting never infers STD from pressure.
* Pressure failure affects only its readout; source/transport loss covers the entire panel.
* Tape backgrounds use the existing opacity token; BARO, text, ticks, and pointer windows remain opaque.
* Accessible labels preserve the full received values or the unavailable/overflow state without per-sample live announcements.

Node and browser tests cover calculations, clips, stable nodes, painted glyph bounds, and state transitions.
Real-phone acceptance remains separate in [#23](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/23).

## Implementation

The altitude module uses the existing right-side allocation and layer 2:

* `altitude/index.js` composes the tape, readout, and independent pressure setting with `resize(layout)` and `render(frame)`.
* The composition owns the shared tape clip, background, unit label, geometry, and local altitude state.
* `tape.js` owns a retained pool of ticks and labels covering the visible range plus edge clearance.
* The pool grows only when resize needs more rows and hides surplus rows on smaller panels.
* Rendering reuses existing nodes without creating or replacing them.
* `readout.js` owns its digit clip, fixed opaque pointer window, sign, rolling columns, and layer-4 X/OVR mark.
* `baro.js` owns its opaque background, cyan border, centered text, and independent state.
* Pure calculations remain in the visual objects that implement them; no generic IAS/altitude tape is introduced.

### Composition boundaries

* Altitude uses its own tape and drums because its scale, signed values, and 20-ft digit steps differ from IAS.
* Selected-altitude and alert rules extend the altitude instrument under #24 without introducing a general tape abstraction.
* The current implementation adds no instrument base class, general tape renderer, or shared lower-indication controller.

### Panel integration

Altitude composes VSI and its trend; heading is a separate instrument.

### Object composition

```text
Altitude
|-- Tape and rolling readout
|-- Pressure setting
|-- Selected altitude and alerts
`-- Vertical speed
```

## Testing

Use the [validation scope matrix](../testing/index.md#validation-by-change-scope) and the feature checks below.
Shared browser commands, phone setup, and evidence rules are linked from those checks.

### Altitude and pressure acceptance

Issue [#23](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/23) owns altitude and pressure evidence.
Use the [altitude contract](../telemetry-contract.md#optional-altitude-and-pressure-setting) and [presentation rules](#presentation).

Automated coverage includes these scenarios:

| Check | Expected behavior |
| --- | --- |
| Rust types and publication | Finite signed feet, verbatim BARO strings, independent omission, shared age, attitude-only clearing, and stale suppression |
| Independent demo fixture | Six altitude sweeps, all segment endpoints, pressure changes, zero crossing, and repeated cycles |
| HTTP/SSE | Both clients receive the same altitude/pressure and sample age; unavailable snapshots omit extensions |
| Node tape geometry | Fixed pixel spacing, known tick positions, wider spans and more labels on taller tapes, signed ticks, and direction |
| Node readouts | 20-ft drums, carries, fractions, signs, and range boundaries |
| Node client and scheduler | Independent field normalization, old snapshots, unchanged callback shape, reconnect/resume, coalescing, and expiry before resize draw |
| Browser fixture | Both compositions, fixed pixel spacing through resize, retained nodes, wider portrait coverage, painted clearance, local failures/overflow, and global recovery |
| Release runtime | Embedded altitude modules with correct MIME, Rust SSE values in SVG/ARIA, numeric BARO presentation, and recovery after restart |

The existing [automated commands](../testing/automated.md#automated-checks) include the altitude Rust and Node tests.
Both [desktop browser scripts](../testing/pfd.md#responsive-efi-layout-and-desktop-checks) are required.
Altitude screenshots appear under `.local/pfd-browser/altitude-*`.
Inspect narrow portrait and short landscape captures, including long signed readouts over sky and ground.

#### Real-phone checks for altitude

Apply the [shared phone procedure](../testing/devices.md#real-phone-acceptance-procedure) for #23, with these altitude-specific checks:

1. Watch a complete cycle for climbing, descending, negative altitude, zero crossing, digit carries, and pressure changes.
2. Check the 100/20-ft markings, fixed pointer, rolling digits, aligned `[KT]`/`[FT]`, and BARO presentation.
3. Compare orientations at the same zoom: tick spacing stays fixed and the taller tape shows more values.
4. Use [phone fixtures](../testing/devices.md#phone-fixture-setup) for independent missing/invalid altitude and pressure, both overflow boundaries, and long signed digits.

#### Tape footers and source-formatted BARO

Issue [#65](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/65) owns the revised footer and BARO acceptance.
The desktop layout script runs `tests/browser/checks/tape-footer-browser-checks.mjs` against both compositions.

| Check | Expected behavior |
| --- | --- |
| Rust, SSE, normalization, and SVG | `23.83 inHg`, `1026 hPa`, and `BARO` reach the readout unchanged |
| Availability and recovery | Missing, null, and non-string data show red `X`; empty and whitespace strings remain available; expiry and recovery follow the common sample deadline |
| Unit labels and scales | Literal `[KT]` and `[FT]` have the same font and visual height, with 1 px above and below desktop glyph bounds; both clips fill their allocated height |
| GS | Label and value share a 13 px font without a suffix; one-, two-, and three-digit values fit without overlap; opaque background matches BARO and the inset frame matches speed-trend magenta |
| BARO | Required examples fit at minimum width, centered in one cyan 11 px text node over opaque dark background with an inset cyan border |
| Background edges | Both 32 px footers meet their tape backgrounds and SVG edges at whole and fractional bounds, including simulated safe-area padding |
| IAS fallback | Four reference values fit between the warning and persistent `[KT]`, without a second unit heading |

Inspect `.local/pfd-browser/footer-*` screenshots at 320×480, 326×246, and 568×240 over both sky and ground.
Run the release-runtime script after building the final embedded Web assets.
For #65, apply the [shared phone procedure](../testing/devices.md#real-phone-acceptance-procedure) and inspect footer readability and edge seams throughout the layout scenarios.

#### Numeric BARO and local unit selection

For [#89](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/89), automated checks cover conversion from one pressure, explicit STD, malformed data, legacy text, both packet orders, expiry, and mode recovery. Browser checks cover a single touch or click, saved preference after reload, unavailable storage, and text fit. A local unit change does not issue simulator commands or renew a sample deadline.

During the combined live acceptance, adjust the cockpit pressure and verify the index 1 indication, enter and leave STD in the simulator, and compare equivalent inHg and hPa after touching the BARO field. Check reload and portrait/landscape readability on available phones using the [shared phone procedure](../testing/devices.md#real-phone-acceptance-procedure). Record which cockpit instrument and devices were actually checked. Desktop and demo results do not establish cockpit synchronization or real-phone acceptance.

### Full-panel geometry

Check altitude rolling digits during full-panel acceptance.

## Rationale and regression lessons

Retain these reasons and validation lessons when changing the presentation:

* Altitude has a separate tape because its responsive span, signed values, and 20-ft drums differ from IAS.
* Keep landscape pixel spacing during resize and reveal more altitude on taller tapes. Do not use a fixed 400-ft span.
* A 24-pixel pressure background caused a 2-pixel text collision in Edge. The corrected height is 32 pixels.
* Check actual pressure glyph bounds and the centered text anchor separately. Font side bearings can produce a subpixel visual offset.
* The #65 BARO replacement deliberately ships server and embedded Web together. Its [versioning exception](../telemetry-contract.md#compatibility) is limited to that replacement.

Evidence limit: real iOS Safari and Android Chrome acceptance remains separate.

### Live BARO follow-up

[Issue #89](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/89): `KOHLSMAN SETTING HG:1` and `KOHLSMAN SETTING STD:1` supply pressure and mode for the same indexed altimeter as indicated altitude. The browser offers local unit selection without simulator writes. Logged pressure does not establish which cockpit control corresponds to this index or prove display synchronization; verify those during live acceptance.
