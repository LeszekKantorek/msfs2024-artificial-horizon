# Panel status and unavailability

The status module controls the global unavailable cover and accessible source/transport announcements.

## Requirements and presentation

Status preserves the full-panel presentation and accessible state:

* The panel has no top status bar or reserved header space.
* The production panel has no source badge or DEMO announcement.
* Only the development fixture overlays a two-line `LAYOUT FIXTURE` badge in the empty tape area above ground speed.
* The fixture badge does not reserve production layout space.
* Successful source and transport messages have no visible labels.
* Source and transport states remain separate internally.
* A polite live region announces only changed text.
* Waiting, reconnecting, stale, suspended, paused, disconnected, and invalid states show an opaque full-panel cover with the applicable reason.
* Only rendering a fresh frame removes the cover.
* A `live` status alone does not remove the cover.

## Implementation

`status.js` presents global availability without owning the connection.

### Application and scheduler integration

* The application passes the same status module instance to the panel.
* `status.update(status)` updates labels from the telemetry callback without exposing old readings.
* Scheduler expiry updates the visible stale reason even if the transport timer has not executed.

### SVG and CSS state

| State written by JavaScript | CSS or presentation effect |
| --- | --- |
| Instrument `data-available` | Global SVG visibility and opaque unavailable cover |

## Testing

Apply the [shared PFD checks](../testing/pfd.md) and [phone lifecycle procedure](../testing/devices.md#real-phone-acceptance-procedure).

### Status acceptance

| Owner | Checks |
| --- | --- |
| #37: status | No top bar, success labels, or DEMO label, with unavailable reasons on an opaque full-panel cover and accessible state changes |
