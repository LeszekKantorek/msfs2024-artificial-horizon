# Frames and rendering lifecycle

Normalized frames, per-sample acceptance, shared scheduling, and expiry.

## Normalized frame contract

| Frame field | Meaning |
| --- | --- |
| `attitude`, `slip_skid`, `turn_rate_dps` | Values from one accepted sample |
| `ias_kt`, `tas_kt`, `gs_kt` | Independently normalized speeds from the same accepted sample, or `null` |
| `airspeed_profile`, `ias_acceleration_ktps` | Independently normalized configuration and source acceleration, or `null` |
| `altitude_ft` | Independently normalized signed altitude, or `null` |
| `baro_setting` | Normalized positive finite pressure, or `null` for missing or invalid data |
| `selected_altitude_ft` | Independently normalized signed selected altitude, or `null` |
| `vertical_speed_fpm` | Independently normalized signed ft/min, or `null` |
| `heading_deg`, `selected_heading_deg` | Independently normalized magnetic degrees in `[0, 360)`, or `null` |
| `expiresAt` | Original monotonic expiry deadline |
| `status` | Copied `source`, `transport`, and `state` values |

Instruments share the frame under these rules:

* Every instrument receives the same complete frame and selects its own fields without mutating the frame or another instrument.
* Nested frame values are also read-only.
* Instruments consume normalized frames.
* Missing or invalid optional readings arrive as `null`, without invented zero values or legacy fallbacks.
* Future feature issues define additions to the frame and wire contract separately.

> The frame is an internal presentation value, not a new SSE schema.

## Panel interface

```js
panel.resize(layout);
panel.accept(frame, now);
panel.render(frame, now);
panel.invalidate(reason);
```

| Operation | Contract |
| --- | --- |
| `resize(layout)` | Update instrument geometry, including while unavailable. Do not renew data validity |
| `accept(frame, now)` | Advance optional instrument history for each accepted fresh sample, without DOM writes |
| `render(frame, now)` | Apply one fresh, coherent presentation frame. Return the earliest next monotonic visual deadline, or `null` |
| `invalidate(reason)` | Obscure invalid indications immediately. Do not substitute level flight or zero values |

`composePanel` delegates optional acceptance and invalidation operations to instruments.
It renders instruments before status and returns the earliest visual deadline from its instruments.
The [instrument composition](composition.md#instrument-composition) defines the hierarchy and extension rules.

## Initialization and geometry updates

* Create instrument elements once during initialization.
* Run `resize(layout)` before the first `render(frame)`.
* Supply real geometry without substitute defaults.

| Component | Initialization | Resize | Render |
| --- | --- | --- | --- |
| Instruments | Create SVG nodes once | Update geometry attributes on existing nodes | Apply pure calculations to SVG using the complete frame |
| Fixed symbols | Create references once | Update reference geometry | No per-frame render method |
| Panel | Compose instrument modules | Assign shared attitude clip, then delegate geometry updates | Delegate the same frame without scale formulas or path construction |

## Sample acceptance and drawing

### Per-sample acceptance

1. Reject an already expired incoming frame through invalidation.
2. If the retained frame has expired, invalidate it before accepting the new frame to reset instrument history.
3. Cancel obsolete timing work and delegate `accept(frame, now)` for each fresh accepted sample without DOM writes.
4. Retain the newest sample and request at most one animation frame.

* Fresh samples with unchanged numeric values still renew freshness.
* Do not interpolate or extrapolate current readings between accepted samples.

### Coalesced frame rendering

The scheduler retains one latest sample and at most one pending draw.
Each animation callback follows this sequence:

1. Check the retained sample's original deadline. Invalidate expired data instead of drawing it.
2. Apply the pending layout.
3. Cancel obsolete timing work and render the fresh frame in the same callback.
4. Schedule the next timed visual update if the panel returns a finite deadline.

### Resize and local preference redraws

| Event | Action | Preserved constraint |
| --- | --- | --- |
| Resize with a retained sample | Keep the newest layout and request at most one animation frame | Apply geometry with fresh readings in the same callback |
| Resize without a retained sample | Apply geometry immediately under the unavailable cover | Do not restore invalid indications |
| BARO unit change | Request a redraw without accepting a new sample | Do not advance per-sample history |

Resize and local preference redraws never renew the sample deadline or advance per-sample history.

## Timed visual updates

The scheduler retains at most one visual timer.

| Owner | Contract |
| --- | --- |
| Instrument render operation | Return a monotonic next-update deadline when needed |
| Panel composition | Select the earliest deadline |
| Scheduler | Accept injectable `now`, `setTimer`, and `clearTimer` dependencies |
| Visual timer | Request the common animation frame at the earlier of the visual deadline and sample expiry |

* Create the visual timer only when rendering returns a finite deadline.
* New samples cancel obsolete timing work.
* Rendering checks the retained sample's original deadline.
* Transport expiry detection continues without new samples through the [telemetry client](telemetry.md#connection-lifecycle-and-freshness).

## Invalidation and recovery

| Condition | Effect |
| --- | --- |
| Optional input failure | Affect only that indication |
| Source/transport loss or sample expiry | Invalidate the panel |
| Reconnect or resume | Wait for a new valid sample before restoring indications |

Invalidation follows this sequence:

1. Cancel timed updates and pending drawing.
2. Discard the retained sample.
3. Invoke `panel.invalidate(reason)` immediately, obscuring invalid indications and resetting instrument state.
4. Apply any pending layout under the unavailable cover.

> Cancellation prevents an apparently valid timed alert from remaining active after invalidation. Do not substitute level flight or zero values.

## Validation invariants

| Behavior | Validation source |
| --- | --- |
| Shared frame, expiry, resize, pause, and recovery | [Shared EFI checks](../testing/pfd.md#responsive-efi-layout-and-desktop-checks) |
| Per-sample history and timed alert cancellation | [Selected-altitude checks](selected-altitude.md#testing) |
| Transport freshness and reconnect | [Client coverage](telemetry.md#client-and-validation-coverage) |

These contracts define expected behavior. Desktop checks do not establish real-device acceptance.
