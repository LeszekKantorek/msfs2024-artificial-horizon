# Altitude and barometric setting

Signed altitude tape and rolling drums, independent BARO, and the altitude composition.

## Requirements

* Display source-supplied altitude for its pressure setting without browser pressure correction.
* Keep altitude and BARO independently optional.
* Show BARO as unavailable when numeric pressure is missing or invalid. Do not use a legacy text fallback.
* Treat finite altitude outside the display range as valid telemetry.

Wire definitions: [telemetry contract](../telemetry-contract.md#optional-altitude-and-pressure-setting).

## Presentation

### Input availability and display limits

Altitude and pressure have independent local states:

| Condition | Presentation |
| --- | --- |
| Altitude from -9999 through 99999 ft | Display the scale and rolling digits |
| Valid altitude outside that range | Show `OVR` and hide the scale and digits |
| Missing altitude | Show `X` and hide the scale and digits. Preserve valid pressure and other instruments |
| Pressure failure | Affect only the BARO readout |
| Source or transport loss | Cover the entire panel |

### Altitude tape and rolling digits

* Center the tape on current altitude, with 20-ft minor ticks and 100-ft major ticks and labels.
* Show higher values above the pointer and cross zero without a direction change.
* On taller tapes, show a wider range and more labels without stretching spacing or text.
* Roll the lower two digits together through 00/20/40/60/80 using the full fractional magnitude.
* Carry higher columns during the final 20 ft before their decimal boundary. Hide leading zeros.
* Display negative values with a separate minus sign and magnitude drums. Zero, including negative zero, has no minus.
* Leave an adjacent drum row outside the display range blank instead of wrapping to zero.

### BARO formatting and local unit selection

BARO presentation uses `baro_setting.pressure_hpa`:

| Input or selection | Display and interaction |
| --- | --- |
| Valid numeric pressure, inHg selected | Two decimal places in inHg |
| Valid numeric pressure, hPa selected | Whole hPa |
| Valid numeric pressure, STD selected | `STD` |
| Missing, null, or invalid numeric pressure | Red `X`. Switching is unavailable, without fallback to legacy text |
| Valid numeric text does not fit | Red `X`; retain the unit control so another selection can display if it fits |

* With valid numeric pressure, a touch or click cycles through inHg, hPa, and STD.
* Persist the browser-local choice across reloads when storage is available. The in-memory choice still works when storage is unavailable.
* A local selection issues no simulator commands and does not renew a sample deadline.

> Local STD does not report cockpit mode. The browser does not infer cockpit STD from pressure.

## Implementation

### Composition and visual object interfaces

The altitude composition uses the existing right-side allocation. Heading remains a separate instrument.

```text
Altitude
|-- Tape and rolling readout
|-- Pressure setting
|-- Selected altitude and alerts
|-- Vertical speed
`-- Altitude trend
```

| Module | Responsibility |
| --- | --- |
| `altitude/index.js` | Compose the visual objects and own the shared tape clip, background, unit label, geometry, and local altitude state |
| `tape.js` | Own the retained pool of ticks and labels for the visible range plus edge clearance |
| `readout.js` | Own the digit clip, fixed opaque pointer window, sign, rolling columns, and layer-4 X/OVR mark |
| `baro.js` | Own the opaque background, cyan border, centered text, and independent pressure state |
| Selected altitude, VSI, and trend | Follow the [selected-altitude](selected-altitude.md), [vertical-speed](vertical-speed.md), and [altitude-trend](altitude-trend.md) contracts |

`createAltitude(instrument, layers, redraw)` exposes these methods:

| Method | Contract |
| --- | --- |
| `resize(layout)` | Set the allocation and local geometry of the composed objects |
| `accept(frame, now)` | Pass altitude and selected altitude to alert sample acceptance |
| `render(frame, now)` | Update local altitude state and the composed indications; return the selected-altitude rendering result |
| `invalidate()` | Invalidate selected altitude, VSI, trend, and BARO |

Composition follows these boundaries:

* Keep pure calculations in the visual objects that implement them.
* Use separate altitude tape and drums because their scale, signed values, and 20-ft digit steps differ from IAS.
* Selected-altitude and alert rules extend the instrument under #24 without introducing a general tape abstraction.
* The current implementation has no instrument base class, general tape renderer, generic IAS/altitude tape, or shared lower-indication controller.

### Geometry, layers, clipping, and paint order

| Parameter | Requirement |
| --- | --- |
| Instrument layer | Existing right-side allocation and layer 2 |
| Warning marks | Layer-4 X/OVR mark |
| Minor tick spacing | 20 ft at fixed 8 CSS px intervals |
| Major tick spacing | 100 ft at fixed 40 CSS px intervals |
| Visible span | 400 ft at 160 CSS px tape height; 1000 ft at 400 CSS px |
| Minimum-height landscape | Match the former 400-ft scale at 160 CSS px tape height |
| BARO allocation | Bottom 32 CSS px of the right column, reaching the panel edge |
| BARO border | One-pixel cyan border inset by half a pixel over an opaque value-window background |
| BARO text | One centered cyan text node, 11 CSS px, weight 600 |
| Opacity | Tape backgrounds use the existing opacity token. BARO, text, ticks, and pointer windows remain opaque |

The composition owns the shared tape clip. The readout owns its digit clip.
The [altitude trend geometry and paint order](altitude-trend.md#geometry-clipping-and-paint-order) defines overlap with ticks, the selected bug, and opaque readouts.

### Frame updates, SVG state, and accessibility

* Grow the tick and label pool only when resize needs more rows. Hide surplus rows on smaller panels.
* Reuse existing nodes during rendering without creating or replacing them.
* Preserve full received values or the unavailable/overflow state in accessible labels, without per-sample live announcements.
* Follow [selected-altitude sample acceptance and scheduling](selected-altitude.md#alert-history-and-scheduler-integration) for alert history and timed rendering.

## Testing

Use the [validation scope matrix](../testing/index.md#validation-by-change-scope) and the feature checks below.
Shared browser commands, phone setup, and evidence rules are linked from those checks.

Issue [#23](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/23) owns altitude and pressure evidence, including separate real-phone acceptance.

### Acceptance scenarios

Use the [altitude contract](../telemetry-contract.md#optional-altitude-and-pressure-setting) and [presentation rules](#presentation).

| Scenario | Expected behavior |
| --- | --- |
| Signed altitude and range boundaries | Correct direction, zero crossing, rolling digits, carries, and local overflow |
| Independent availability | Missing altitude preserves valid BARO; pressure failure preserves altitude |
| Resize and recovery | Fixed tick spacing, wider coverage on taller tapes, retained nodes, and recovery after local or global loss |
| Numeric BARO | Apply local unit selection, text-fit handling, and independent availability without legacy fallbacks |

### Automated checks and independent fixtures

The existing [automated commands](../testing/automated.md#automated-checks) include the altitude Rust and Node tests.
Node and browser tests cover calculations, clips, stable nodes, painted glyph bounds, and state transitions.

| Check | Expected behavior |
| --- | --- |
| Rust types and publication | Finite signed feet, positive finite pressure, independent omission, shared age, attitude-only clearing, and stale suppression |
| Independent demo fixture | Six altitude sweeps, all segment endpoints, pressure changes, zero crossing, and repeated cycles |
| Node tape geometry | Fixed pixel spacing, known tick positions, wider spans and more labels on taller tapes, signed ticks, and direction |
| Node readouts | 20-ft drums, carries, fractions, signs, and range boundaries |
| Node client and scheduler | Independent field normalization, old snapshots, unchanged callback shape, reconnect/resume, coalescing, and expiry before resize draw |

Verify numeric BARO and local selection:

| Check | Coverage |
| --- | --- |
| Automated calculations and state | Conversion from one pressure, missing and malformed data, expiry, and recovery |
| Browser interaction | Local inHg/hPa/STD cycle, saved preference after reload, unavailable storage, and text fit |
| Selection boundary | No simulator commands and no renewal of the sample deadline |

### Desktop geometry and availability transitions

Both [desktop browser scripts](../testing/pfd.md#responsive-efi-layout-and-desktop-checks) are required.

Browser fixtures cover both compositions:

* Preserve fixed pixel spacing through resize and retain nodes.
* Show wider portrait coverage and painted clearance.
* Exercise local failures and overflow, followed by global recovery.

Inspect `.local/pfd-browser/altitude-*` screenshots in narrow portrait and short landscape, including long signed readouts over sky and ground.

#### Tape footers and BARO

The desktop layout script runs `tests/browser/checks/tape-footer-browser-checks.mjs` against both compositions.

| Check | Expected behavior |
| --- | --- |
| Rust, SSE, normalization, and SVG | Source pressure reaches the readout through `baro_setting`. The local unit determines formatting |
| Availability and recovery | Missing, null, or invalid pressure shows red `X` without legacy fallback. Expiry and recovery follow the common sample deadline |
| Unit labels and scales | Literal `[KT]` and `[FT]` have the same font and visual height, with 1 px above and below desktop glyph bounds; both clips fill their allocated height |
| GS | Label and value share a 13 px font without a suffix; one-, two-, and three-digit values fit without overlap; opaque background matches BARO and the inset frame matches speed-trend magenta |
| BARO | Formatted inHg, hPa, and STD readings fit at minimum width, centered in one cyan 11 px text node over opaque dark background with an inset cyan border |
| Background edges | Both 32 px footers meet their tape backgrounds and SVG edges at whole and fractional bounds, including simulated safe-area padding |
| IAS fallback | Four reference values fit between the warning and persistent `[KT]`, without a second unit heading |

Inspect `.local/pfd-browser/footer-*` screenshots at 320×480, 326×246, and 568×240 over both sky and ground.

### Real-phone checks

Apply the [shared phone procedure](../testing/devices.md#real-phone-acceptance-procedure) for #23, with these altitude-specific checks:

1. Watch a complete cycle for climbing, descending, negative altitude, zero crossing, digit carries, and pressure changes.
2. Check the 100/20-ft markings, fixed pointer, rolling digits, aligned `[KT]`/`[FT]`, and BARO presentation.
3. Compare orientations at the same zoom: tick spacing stays fixed and the taller tape shows more values.
4. Use [phone fixtures](../testing/devices.md#phone-fixture-setup) for independent missing/invalid altitude and pressure, both overflow boundaries, and long signed digits.

Additional phone checks retain their acceptance scope:

| Scope | Checks |
| --- | --- |
| #65 | Inspect footer readability and edge seams throughout the shared phone layout scenarios |
| #89 | Check reload and portrait/landscape readability on available phones. Record the devices actually checked |

> Real iOS Safari and Android Chrome acceptance remains separate from desktop checks.

### Live BARO acceptance

For #89, `KOHLSMAN SETTING HG:1` supplies pressure for the same indexed altimeter as indicated altitude.
Validate the source and local selection against the cockpit:

1. Adjust cockpit pressure and verify the index 1 indication, including which cockpit control corresponds to that index.
2. Touch the BARO field and compare equivalent inHg and hPa values.
3. Verify that the locally selected STD label does not command the cockpit or report its mode.
4. Apply the [real-phone checks](#real-phone-checks) for reload and portrait/landscape readability.
5. Record which cockpit instrument and devices were actually checked.

> Logged pressure does not establish the corresponding cockpit control or prove display synchronization. Desktop and demo results do not establish cockpit synchronization or real-phone acceptance.

### Full-panel integration

| Check | Expected behavior |
| --- | --- |
| HTTP/SSE | Both clients receive the same altitude/pressure and sample age; unavailable snapshots omit extensions |
| Release runtime | Embedded altitude modules with correct MIME, Rust SSE values in SVG/ARIA, numeric BARO presentation, and recovery after restart |

* Run the release-runtime script after building the final embedded Web assets, including the #65 footer and BARO checks.
* Check altitude rolling digits during full-panel acceptance.

## Rationale and regression lessons

Retain these reasons and validation lessons when changing the presentation:

* Altitude has a separate tape because its responsive span, signed values, and 20-ft drums differ from IAS.
* Keep landscape pixel spacing during resize and reveal more altitude on taller tapes. Do not use a fixed 400-ft span.
* A 24-pixel pressure background caused a 2-pixel text collision in Edge. The corrected height is 32 pixels.
* Check actual pressure glyph bounds and the centered text anchor separately. Font side bearings can produce a subpixel visual offset.
* Server, telemetry contract, and embedded Web follow the [application delivery contract](../architecture.md#application-delivery-contract). Backward compatibility is not a project requirement.
