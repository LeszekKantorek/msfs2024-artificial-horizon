# Panel status and unavailability

The status module controls the global unavailable cover and accessible source/transport announcements.

## Requirements

* Keep the panel free of a top status bar and reserved header space.
* Show no source badge or DEMO announcement in the production panel.
* Keep source and transport states separate internally.
* Present global availability without owning the connection.
* Remove the cover only when rendering a fresh frame, never from a `live` status alone.

## Presentation

### Global cover and visible labels

| State or event | Visible result |
| --- | --- |
| Waiting, reconnecting, stale, suspended, paused, disconnected, or invalid | Opaque full-panel cover with the applicable reason |
| Successful source or transport message | No visible success label |
| `live` status without a fresh rendered frame | Preserve the cover without exposing old readings |
| Rendering a fresh frame | Remove the cover |

The development fixture has its own badge:

* Overlay a two-line `LAYOUT FIXTURE` badge in the empty tape area above ground speed.
* Show this badge only in the development fixture.
* Reserve no production layout space for the badge.

### Accessible announcements

* Use a polite live region for source and transport announcements.
* Update its text only when the announcement changes, rather than for every telemetry sample.
* Keep accessible state changes available without visible success labels.

## Implementation

### Public interface and component responsibilities

`createStatusView(document)` in `status.js` returns these operations:

| Operation | Contract |
| --- | --- |
| `update(status)` | Copy the current status, update source/transport/state attributes, and announce changed text. Do not change panel availability |
| `render(frame)` | Update status from `frame.status`, then set panel availability to true |
| `invalidate(reason)` | Set availability to false, update the cover reason, and update the accessible announcement |

The integration keeps these responsibilities separate:

| Component | Responsibility |
| --- | --- |
| Application | Pass the same status module instance to the panel. Call `status.update(status)` from the telemetry callback without exposing old readings |
| Panel | Render instrument updates before status. Delegate invalidation to status and the instruments |
| Scheduler | Enforce frame freshness and expiry through the panel lifecycle |
| Status view | Present availability and source/transport messages without owning the connection or validating frame freshness |

The [frame lifecycle](lifecycle.md#normalized-frame-contract) supplies the fresh-frame contract.
`render(frame)` relies on that contract. It does not independently check the sample deadline.

### Frame updates and DOM state

Scheduler expiry updates the visible stale reason even if the transport timer has not executed.
The status view also updates its stored source state and accessible announcement for that reason, keeping them consistent with the cover.

| Attribute or element | Written state | Effect or use |
| --- | --- | --- |
| Instrument `data-available` | True after rendering; false on invalidation | Control global SVG visibility and the opaque unavailable cover |
| Instrument `data-source` | Current source, or `unavailable` when absent | Preserve source identity separately from transport and source state |
| Instrument `data-transport` | Current transport state | Preserve transport state |
| Instrument `data-state` | Current source state, including source-state invalidation reasons | Preserve source state |
| `#unavailable` | Applicable source or transport reason | Display the reason on the cover |
| `#telemetry-status` | Source and transport announcement, or the reason before a current status exists | Announce changed text through the polite live region |

## Testing

Apply the [shared EFI checks](../testing/pfd.md).
Issue #37 owns status acceptance, including visible presentation and accessible state changes.

### Acceptance scenarios

These criteria define expected behavior. They do not claim separate automated coverage for every row:

| Scenario | Expected result |
| --- | --- |
| Production panel | No top bar, reserved header space, source badge, visible success labels, or DEMO announcement |
| Development fixture | Two-line `LAYOUT FIXTURE` badge above ground speed, without reserving production layout space |
| Unavailable source or transport | Opaque full-panel cover with the applicable reason |
| `live` status before rendering a fresh frame | Cover remains and old readings stay hidden |
| Fresh frame rendered | Cover disappears after instrument updates |
| Scheduler expiry before the transport timer | Visible stale reason and accessible source state update consistently |
| Repeated identical announcement | No repeated text update in the live region |
| Changed source or transport announcement | Polite live region receives the changed text |

### Real-phone checks

Apply the [phone lifecycle procedure](../testing/devices.md#real-phone-acceptance-procedure) and the status acceptance scenarios.
Record the actual device results for #37.

> Desktop checks do not establish real iOS Safari or Android Chrome acceptance. The scenarios describe required checks, not completed device evidence.
