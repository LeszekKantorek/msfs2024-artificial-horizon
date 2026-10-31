# Attitude, fixed references, slip/skid, and turn rate

The horizon and fixed visual references, with independent coordinated-turn indications.

## Requirements

* Use finite pitch in `[-90, 90]` and roll in `[-180, 180)`.
* Keep slip/skid and turn rate independently optional and use the shared sample deadline.
* Hide only the affected moving indication and show its local unavailable mark when an optional value is missing.
* Never replace missing attitude with level flight.

Wire definitions: [attitude fields and conventions](../telemetry-contract.md#fields-and-conventions) and [optional coordinated-turn indications](../telemetry-contract.md#optional-coordinated-turn-indications).

## Presentation

### Input availability and display limits

| Input | Range or availability rule |
| --- | --- |
| Pitch | Finite degrees in `[-90, 90]` |
| Roll | Finite degrees in `[-180, 180)` |
| Slip/skid | Normalized displacement from -1 left to +1 right, with zero centered |
| Turn rate | Display span of +/-6 deg/s; values beyond it use edge arrows |
| Missing or invalid slip/skid or turn rate | Hide only its moving indication and show a local unavailable mark |
| Missing attitude | Never substitute level flight |

### Horizon, pitch scale, and fixed references

* Nose up lowers the horizon. Right bank rotates the world counterclockwise around the fixed aircraft reference.
* Draw sky and ground across the full SVG, with central clipping for pitch markings.
* Keep fixed aircraft references stationary.
* Use 2.5-degree pitch intervals and red chevrons beginning at +60/-40 scale positions, pointing toward the horizon.
* Preserve undistorted symbols and the shared attitude clip through resize and invalidation.

### Slip/skid and turn rate

| Indication | Presentation |
| --- | --- |
| Slip/skid | Zero is centered. Negative displacement moves left and positive displacement moves right |
| Turn rate | Standard-rate marks at +/-3 deg/s within the +/-6 deg/s display span |
| Turn rate beyond the display span | Show an edge arrow |

## Implementation

### Composition and visual object interfaces

| Module | Responsibility |
| --- | --- |
| `horizon/` | Compose world, pitch scale with chevrons, and bank pointer. Calculate common attitude transforms |
| `fixed-symbols/` | Compose fixed bank scale and aircraft reference with outline |
| `slip-skid.js`, `turn-rate.js` | Local geometry, readings, and indication availability |

```text
Horizon
|-- World (sky, ground, horizon line)
|-- Pitch scale and chevrons
`-- Bank pointer

Fixed symbols
|-- Bank scale
`-- Aircraft reference and outline
```

The instrument factories expose these operations:

| Factory | Returned operations |
| --- | --- |
| `createHorizon(instrument, layers)` in `horizon/index.js` | `resize(layout)`, `render(frame)` |
| `createFixedSymbols(instrument, symbols)` in `fixed-symbols/index.js` | `resize(layout)` only |
| `createSlipSkid(instrument, layers)` in `slip-skid.js` | `resize(layout)`, `render(frame)` |
| `createTurnRate(instrument, layers)` in `turn-rate.js` | `resize(layout)`, `render(frame)` |

The composed visual objects retain concrete inputs and ownership:

| Object / factory | Owned content | Operations and concrete inputs |
| --- | --- | --- |
| World: `createWorld(rotationHost)` | World rotation/pitch hosts and sky/ground/horizon paths | `resize({ width, height, pitchScale })`, `render(transforms)` |
| Pitch: `createPitchScale(scaleRotationHost, warningRotationHost)` | Scale/warning rotation/pitch hosts, ticks, labels, chevrons | `resize({ pitchScale, labelSize })`, `render(transforms)` |
| Pointer: `createBankPointer(originHost)` | Exclusive pointer origin and path | `resize({ cx, cy, bankRadius })`, `render(transforms)` |
| Fixed composition | Common fixed-symbol origin and layer mounting | `resize(layout)` |
| Bank: `createBankScale(bankHost)` | Bank zero mark, ticks, and local constants | `resize(radius)` |
| Aircraft: `createAircraft(fixedHost)` | Aircraft shape/outline and common path calculation | `resize(width)` |

Panel delegation preserves these boundaries:

* Horizon owns the common world/warning origins, background clip, and pitch clip.
* The panel owns `#attitude-clip`, shared by horizon's bank pointer and fixed symbols.
* `composePanel(svg, instruments, status)` retains its API.
* Panel composition assigns `layout.attitude` before instrument resize.
* No instrument scale or path calculation moves into panel delegation.

### World transforms

`horizon/index.js` owns `attitudeTransforms` and calculates the transforms once per frame from attitude and initialized local pitch geometry.
World, pitch scale, and bank pointer receive the same transforms object.

Pitch translation remains nested inside bank rotation. Background, attitude, and pitch clips stay fixed while the world rotates and translates.

### Geometry, layers, clipping, and paint order

| Element | Layer / origin | Clip and owner |
| --- | --- | --- |
| Sky/ground | Horizon, layer 0, origin at attitude center | Horizon background clip uses `layout.background` across the full SVG |
| Pitch scale | Horizon, layer 0, origin at attitude center | Horizon pitch clip fixed relative to horizon origin |
| Bank pointer | Horizon, layer 3, origin at attitude center | Panel attitude clip in panel coordinates |
| Chevrons | Horizon, layer 4, origin at attitude center | Horizon pitch clip fixed relative to horizon origin |
| Fixed symbols | Layer 3, origin at attitude center | Same panel attitude clip uses `layout.attitude` |
| Slip/skid and turn rate | Own translated origins | Panel bounds |

Paint fixed children in this order:

1. Bank scale.
2. Aircraft outline.
3. Aircraft shape.

### Frame updates and SVG state

* Fixed objects have no per-frame render operation.
* Aircraft calculates its common shape/outline path once per resize.
* JavaScript writes slip/skid and turn-rate `data-available` to control local unavailable marks and hide the affected indication.

## Testing

* Run both [desktop browser scripts](../testing/pfd.md#responsive-efi-layout-and-desktop-checks) for presentation changes.
* Verify that unregistered asset paths return `404`, without aliases, under the [HTTP route contract](../server.md#http-routes).
* Use [controlled fixtures](../testing/pfd.md#controlled-browser-fixtures) for local turn failures, extreme poses, expiry, and recovery.

### Acceptance scenarios

| Scenario | Expected result |
| --- | --- |
| Nose up | Horizon moves downward |
| Positive right bank | World rotates counterclockwise around the fixed aircraft reference |
| Combined pitch/bank | Fixtures detect transform-order errors |
| Pitch scale and warnings | 2.5-degree intervals, with red chevrons starting at +60/-40 scale positions and pointing toward the horizon |
| Slip/skid | Center and both directions, with a local unavailable mark for missing or invalid data |
| Turn rate | +/-3 deg/s reaches the standard-rate marks, with an edge arrow beyond +/-6 deg/s |
| Pitch +/-90 degrees, roll -180 / 179.999 degrees | No uncovered background |

### Automated checks and independent fixtures

Calculation and lifecycle suites cover these boundaries:

| Suite | Coverage / method |
| --- | --- |
| Horizon | `tests/horizon.test.mjs` imports `attitudeTransforms` from `web/horizon/index.js` for independent pose signs and extreme/default transforms |
| Coordinated turn | Typed bounds, independent optional failures, current attitude-only publications, shared sample age, +/-3 deg/s, resize without refreshing age |

`tests/horizon.test.mjs` checks independent pose signs and combined/extreme transforms without a DOM.
Independent fixtures retain separate scopes:

| Fixture / suite | Coverage |
| --- | --- |
| `tests/fixtures/attitudes.json` | Independent degree/sign expectations for all seven demo poses |
| `tests/fixtures/pfd-samples.json` | Nine independent extended samples, repeated every 360 publications |

### Desktop geometry and availability transitions

* Check pitch geometry immediately after `ResizeObserver`, before the next animation callback.
* Check matching world/pitch/chevron transforms, fixed aircraft/outline paths, and original SVG hierarchy across poses and resize.
* Module regression checks cover presentation, pose signs, combined transforms, slip/skid, turn rate, and local unavailable indications.

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

### Real-phone checks

Apply the [shared phone procedure](../testing/devices.md#real-phone-acceptance-procedure) and record the required results.
Add these attitude checks for each phone:

* [ ] Check that no DEMO label or top status bar appears.
* [ ] Observe the full current cycle and indications listed in [Demo behavior](#demo-behavior).
* [ ] Check that nose up lowers the horizon and right bank rotates it counterclockwise.

Complete coordinated-turn and attitude acceptance on each device:

1. Run the real-device procedure on iOS Safari and Android Chrome.
2. Include centered/left/right slip, both standard-rate turns, and both chevron scenarios.
3. Exercise orientation, browser bars, lock/resume, Wi-Fi recovery, and server recovery.
4. Record device, OS/browser, build, and network details with the acceptance results.

> Desktop checks do not replace acceptance on actual iOS Safari and Android Chrome. Keep missing device evidence explicit.

### Simulator comparison scope

When checking simulator compatibility on actual devices, compare these attitudes with the known demo pose and simulator cockpit:

* Level.
* Positive and negative pitch.
* Positive and negative bank.
* Combined pitch and bank.

The expected result is agreement with the known demo pose and simulator cockpit.
These are required comparisons, not a record of completed simulator or device acceptance.

### Full-panel integration

Check pitch/bank signs and extremes, slip/skid in both directions, and +/-3 deg/s turns during full-panel acceptance.

#### Demo behavior

Use the [Demo attitude and coordinated-turn schedule](../sources/demo.md#attitude-and-coordinated-turn) for exact source values.
The cycle exercises these presentation cases:

| Property | Expected behavior |
| --- | --- |
| Cycle | Nine poses over 18 seconds, each held for two seconds |
| Poses | Level, nose up/down, left/right bank, two combined attitudes, then +70/-50 pitch |
| Coordinated turn | Centered and both-direction slip/skid, with coordinated and uncoordinated +/-3 deg/s turns |

## Rationale and regression lessons

* Keep pitch translation inside bank rotation. Combined-pose fixtures detect incorrect transform order.
* Preserve the fixed clip boundaries while the world moves, and keep instrument calculations out of panel delegation.

Bank-zero clipping was reproduced at a 568 × 240 desktop viewport and is covered by `tests/layout.test.mjs`.
This recorded regression establishes desktop geometry evidence, not phone acceptance.
