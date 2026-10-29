# Frames and rendering lifecycle

Normalized frames, per-sample acceptance, shared scheduling, and expiry.

## Panel interface and frame lifecycle

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

| Frame field | Meaning |
| --- | --- |
| `attitude`, `slip_skid`, `turn_rate_dps` | Values from one accepted sample |
| `ias_kt`, `tas_kt`, `gs_kt` | Independently normalized speeds from the same accepted sample, or `null` |
| `airspeed_profile`, `ias_acceleration_ktps` | Independently normalized configuration and source acceleration, or `null` |
| `altitude_ft` | Independently normalized signed altitude, or `null` |
| `baro_text` | Verbatim source-formatted display string, or `null` for missing/non-string data |
| `baro_setting` | Omitted when absent on the wire; otherwise normalized pressure, or `null` when invalid. Present invalid data takes precedence over legacy text |
| `selected_altitude_ft` | Independently normalized signed selected altitude, or `null` |
| `vertical_speed_fpm` | Independently normalized signed ft/min, or `null` |
| `heading_deg`, `selected_heading_deg` | Independently normalized magnetic degrees in `[0, 360)`, or `null` |
| `expiresAt` | Original monotonic expiry deadline |
| `status` | Copied `source`, `transport`, and `state` values |

Instruments share the frame under these rules:

* Every instrument receives the same complete frame and selects its own fields without mutating the frame or another instrument.
* Nested frame values are also read-only.
* Instruments consume normalized frames.
* Missing or invalid optional readings arrive as `null`, without invented zero values, except that `baro_setting` preserves wire-field presence for legacy compatibility.
* `resize(layout)` must run before the first `render(frame)`.
* Initialization supplies real geometry without substitute defaults.
* Future feature issues define additions to the frame and wire contract separately.

> The frame is an internal presentation value, not a new SSE schema.

The [module composition](composition.md#module-composition) shows the instrument hierarchy.
`composePanel` delegates optional acceptance and invalidation operations to those instruments.
During rendering, it delegates instrument updates before rendering status.
Panel rendering returns the earliest visual deadline from its instruments.

| Component | Initialization | Resize | Render |
| --- | --- | --- | --- |
| Instruments | Create SVG nodes once | Update geometry attributes on existing nodes | Apply pure calculations to SVG using the complete frame |
| Fixed symbols | Create references once | Update reference geometry | No per-frame render method |
| Panel | Compose instrument modules | Assign shared attitude clip, then delegate geometry updates | Delegate the same frame without scale formulas or path construction |

* Adding a data dependency inside an instrument does not change its call in the panel.
* Adding an instrument requires explicit composition in the panel.

The scheduler coordinates samples, layout, and invalidation in this sequence:

1. Create instrument elements once during initialization.
2. On each fresh accepted sample, delegate `accept(frame, now)` before retaining the newest sample and requesting at most one animation frame.
3. On resize with a retained sample, keep the newest layout and request at most one animation frame. A BARO unit change can also request a redraw without accepting a new sample.
4. Before drawing, check the original deadline, then apply the pending layout and render the fresh frame in the same callback.
5. On invalidation, cancel pending drawing, discard the sample, and invoke `panel.invalidate(reason)` immediately.
6. Apply any pending layout under the unavailable cover, updating geometry immediately when no sample remains.
7. After reconnect or resume, wait for a new valid sample before restoring indications.

Freshness and status handling follow these rules:

* Resize and local preference redraws never renew the sample deadline or advance per-sample history.
* Transport expiry detection continues even when no new samples arrive.
* Fresh samples with unchanged numeric values still renew freshness.
* Optional failures affect only their indications.
* Source/transport loss invalidates the panel.
* Do not interpolate or extrapolate current readings between accepted samples.

Timed visual changes use the shared scheduling and cancellation path:

* Instrument render operations can return a monotonic next-update deadline.
* Panel composition selects the earliest.
* The scheduler accepts injectable `now`, `setTimer`, and `clearTimer` dependencies.
* One timer requests the common animation frame at the earlier of the visual deadline and sample expiry.
* New samples cancel obsolete timing work.
* Rendering checks the retained sample's original deadline.
* Invalidation cancels their pending work and prevents an apparently valid alert from remaining active.
