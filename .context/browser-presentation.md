---
description: Extend instrument rendering or prepare real-phone acceptance.
status: active
---

* Use original assets for the dark, rectangular, G5-inspired SVG instrument.
* Render the latest accepted sample on the next animation frame without interpolation.
* Reconsider smoothing only after observing mobile behavior.
* Separate source and transport status.
* Obscure unavailable attitude instead of displaying a credible level-flight indication.
* Use [PFD scope](pfd-demo-scope.md) for instrument scope and extension constraints.
* Reuse the [recorded phone environments](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/4#issuecomment-6065793741) when later feedback explicitly states that the same devices were used.

| Evidence | Limit |
| --- | --- |
| Desktop viewports / synthetic SSE | Browser behavior only, not phone lifecycle or simulator compatibility |
| Real iOS Safari / Android Chrome | Run the documented procedures on actual devices. Record the tested platform (iOS or Android), behavior, and result |

Procedures: [testing](../docs/testing/index.md). Freshness/status: [contract](../docs/telemetry-contract.md).

* Follow the [mobile acceptance record rules](../docs/testing/devices.md#mobile-acceptance-records) when recording phone checks.

* Fill the entire panel with sky/ground and retain a separate central pitch clip.
* Do not show a top status bar or DEMO label. Keep unavailable reasons on the full-panel cover.
* Use the accepted background opacity of `0.65`. Follow the [opacity checks](../docs/efi/rendering.md#real-phone-opacity-acceptance) when changing background presentation.

Implementation details belong in [panel presentation](../docs/efi/index.md).

The development-only LAYOUT FIXTURE marker remains.
