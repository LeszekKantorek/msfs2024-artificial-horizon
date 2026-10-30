# Attitude, fixed references, slip/skid, and turn rate

The horizon and fixed visual references, with independent coordinated-turn indications.

## Requirements

Use finite pitch in `[-90, 90]` and roll in `[-180, 180)`.
Nose up lowers the horizon. Right bank rotates the world counterclockwise.
Slip/skid and turn rate are independently optional and share the sample deadline. Missing values hide only their moving indication and show a local unavailable mark.
Never replace missing attitude with level flight.

Wire definitions: [attitude fields and conventions](../telemetry-contract.md#fields-and-conventions) and [optional coordinated-turn indications](../telemetry-contract.md#optional-coordinated-turn-indications).

## Presentation

The attitude and turn indications use these display rules:

* Draw sky and ground across the full SVG, with central clipping for pitch markings.
* Keep pitch translation nested inside bank rotation and fixed aircraft references stationary.
* Use 2.5-degree pitch intervals and red chevrons beginning at +60/-40 scale positions, pointing toward the horizon.
* Slip/skid uses normalized displacement from -1 left to +1 right, with zero centered.
* Turn rate spans +/-6 deg/s, with standard-rate marks at +/-3 deg/s and edge arrows beyond the display range.
* Preserve undistorted symbols and the shared attitude clip through resize and invalidation.

## Implementation

Horizon shares origins, clips, and transforms through composition:

* Horizon owns the common world/warning origins, background clip, and pitch clip.
* `horizon/index.js` owns `attitudeTransforms` and calculates the transforms once per frame from attitude and initialized local pitch geometry.
* World, pitch scale, and bank pointer receive the same transforms object.

| Object / factory | Owned content | Operations and concrete inputs |
| --- | --- | --- |
| World: `createWorld(rotationHost)` | World rotation/pitch hosts and sky/ground/horizon paths | `resize({ width, height, pitchScale })`, `render(transforms)` |
| Pitch: `createPitchScale(scaleRotationHost, warningRotationHost)` | Scale/warning rotation/pitch hosts, ticks, labels, chevrons | `resize({ pitchScale, labelSize })`, `render(transforms)` |
| Pointer: `createBankPointer(originHost)` | Exclusive pointer origin and path | `resize({ cx, cy, bankRadius })`, `render(transforms)` |
| Fixed composition | Common fixed-symbol origin and layer mounting | `resize(layout)` |
| Bank: `createBankScale(bankHost)` | Bank zero mark, ticks, and local constants | `resize(radius)` |
| Aircraft: `createAircraft(fixedHost)` | Aircraft shape/outline and common path calculation | `resize(width)` |

Panel and fixed-object boundaries remain explicit:

* The panel owns `#attitude-clip`, shared by horizon's bank pointer and fixed symbols.
* `composePanel(svg, instruments, status)` retains its API.
* Panel composition assigns existing `layout.attitude` before instrument resize.
* No instrument scale or path calculation moves into panel delegation.
* Fixed objects have no per-frame render operation.
* Aircraft calculates its common shape/outline path once per resize.

### Factory interface

| Factory | Returned operations |
| --- | --- |
| `createHorizon(instrument, layers)` in `horizon/index.js` | `resize(layout)`, `render(frame)` |
| `createFixedSymbols(instrument, symbols)` in `fixed-symbols/index.js` | `resize(layout)` only |

### Module responsibilities

| Module | Responsibility |
| --- | --- |
| `horizon/` | Compose world, pitch scale with chevrons, and bank pointer. Calculate common attitude transforms |
| `fixed-symbols/` | Compose fixed bank scale and aircraft reference with outline |
| `slip-skid.js`, `turn-rate.js` | Local geometry, readings, and indication availability |

### Object composition

```text
Horizon
|-- World (sky, ground, horizon line)
|-- Pitch scale and chevrons
`-- Bank pointer

Fixed symbols
|-- Bank scale
`-- Aircraft reference and outline
```

### Layers and local clips

| Element | Layer / origin | Clip and owner |
| --- | --- | --- |
| Sky/ground | Horizon, layer 0, origin at attitude center | Horizon background clip uses `layout.background` across the full SVG |
| Pitch scale | Horizon, layer 0, origin at attitude center | Horizon pitch clip fixed relative to horizon origin |
| Bank pointer | Horizon, layer 3, origin at attitude center | Panel attitude clip in panel coordinates |
| Chevrons | Horizon, layer 4, origin at attitude center | Horizon pitch clip fixed relative to horizon origin |
| Fixed symbols | Layer 3, origin at attitude center | Same panel attitude clip uses `layout.attitude` |
| Slip/skid and turn rate | Own translated origins | Panel bounds |

### World transforms

* Background, attitude, and pitch clips stay fixed while the world rotates and translates.
* Pitch translation remains nested inside bank rotation.

### SVG paint order

| Scope | Order |
| --- | --- |
| Fixed children | Bank scale, aircraft outline, aircraft shape |

### SVG and CSS state

| State written by JavaScript | CSS or presentation effect |
| --- | --- |
| Slip/skid and turn-rate `data-available` | Local unavailable marks and hiding of the affected indication |

## Testing

`tests/horizon.test.mjs` checks independent pose signs and combined/extreme transforms without a DOM.
Run both [desktop browser scripts](../testing/pfd.md#responsive-efi-layout-and-desktop-checks) for presentation changes.
Use [controlled fixtures](../testing/pfd.md#controlled-browser-fixtures) for local turn failures, extreme poses, expiry, and recovery.

### Attitude display: real-device phone acceptance

Apply the [shared phone procedure](../testing/devices.md#real-phone-acceptance-procedure) and record results in #4.
Add these attitude checks for each phone:

* [ ] Check that no DEMO label or top status bar appears.
* [ ] Observe the full current cycle and indications listed in [Demo behavior](#demo-behavior).
* [ ] Check that nose up lowers the horizon and right bank rotates it counterclockwise.

### Coordinated-turn and attitude extension acceptance

Complete the separate phone acceptance for #20:

1. Run the real-device procedure on iOS Safari and Android Chrome for #20.
2. Include centered/left/right slip, both standard-rate turns, both chevron scenarios, orientation, browser bars, lock/resume, Wi-Fi and server recovery.
3. Record device, OS/browser, build and network details in the issue.
4. Keep #20 open while this hardware evidence is missing.

### Geometry regression expectations

* Nose up lowers the horizon.
* Positive right bank rotates the world counterclockwise around the fixed aircraft reference.
* Combined pitch/bank fixtures detect transform-order errors.

### Focused calculation and lifecycle suites

| Suite | Coverage / method |
| --- | --- |
| Horizon | `tests/horizon.test.mjs` imports `attitudeTransforms` from `web/horizon/index.js` for independent pose signs and extreme/default transforms |
| Coordinated turn | Typed bounds, independent optional failures, old snapshots, shared sample age, +/-3 deg/s, resize without refreshing age |

### Independent fixtures

| Fixture / suite | Coverage |
| --- | --- |
| `tests/fixtures/attitudes.json` | Independent degree/sign expectations for all seven demo poses |
| `tests/fixtures/pfd-samples.json` | Nine independent extended samples, repeated every 360 publications |

### Demo behavior

Use the [Demo attitude and coordinated-turn schedule](../sources/demo.md#attitude-and-coordinated-turn) for exact source values.
The cycle exercises these presentation cases:

| Property | Expected behavior |
| --- | --- |
| Cycle | Nine poses over 18 seconds, each held for two seconds |
| Poses | Level, nose up/down, left/right bank, two combined attitudes, then +70/-50 pitch |
| Coordinated turn | Centered and both-direction slip/skid, with coordinated and uncoordinated +/-3 deg/s turns |

### Desktop indication checks

| Check | Expected result |
| --- | --- |
| Pitch scale / warnings | 2.5-degree intervals, with red chevrons starting at +60/-40 scale positions and pointing toward the horizon |
| Slip/skid | Center and both directions, with a local unavailable mark for missing or invalid data |
| Turn rate | +/-3 deg/s reaches the standard-rate marks, with an edge arrow beyond +/-6 deg/s |

### Resize and transform regressions

* Check pitch geometry immediately after `ResizeObserver`, before the next animation callback.
* Check matching world/pitch/chevron transforms, fixed aircraft/outline paths, and original SVG hierarchy across poses and resize.

### Extreme-pose coverage

| Scenario | Check |
| --- | --- |
| Pitch +/-90 degrees, roll -180 / 179.999 degrees | No uncovered background |

### Controlled attitude and turn inputs

Start the [shared fixture server](../testing/pfd.md#controlled-browser-fixtures), then enter these JSON lines:

```json
{"state":"live","pitch_deg":10,"roll_deg":25,"hold":false,"malformed":false}
{"state":"live","slip_skid":-1,"turn_rate_dps":-3}
{"slip_skid":1,"turn_rate_dps":3}
{"slip_skid":"bad","turn_rate_dps":3}
{"slip_skid":0,"turn_rate_dps":"bad"}
{"attitude_only":true}
{"attitude_only":false,"slip_skid":0,"turn_rate_dps":0}
```

### Composition image regression

| Owner | Checks |
| --- | --- |
| #36: modules | Preserve the current image, pose signs, combined transforms, slip/skid, turn rate, and local unavailable indications |

### Retired asset routes

| Retired route | Current owner |
| --- | --- |
| `/horizon.js`, `/fixed-symbols.js` | Their feature directories |
| `/horizon/calculations.js` | Pure functions in the implementing visual objects |

### Full-panel attitude scenarios

| Area | Scenarios and acceptance |
| --- | --- |
| Attitude/turn geometry | Pitch/bank signs and extremes, slip/skid both ways, +/-3 deg/s turns |

### Geometry evidence provenance

Bank-zero clipping was reproduced at a 568 × 240 desktop viewport and is covered by `tests/layout.test.mjs`.

### Simulator comparison scope

Apply this comparison when checking simulator compatibility on actual devices:

| Scenario | Expected result |
| --- | --- |
| Level / +/- pitch / +/- bank / combined | Agreement with known demo pose and simulator cockpit |
