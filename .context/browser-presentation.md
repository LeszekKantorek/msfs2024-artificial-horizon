---
description: Extend instrument rendering or prepare real-phone acceptance.
status: active
---

* Use original assets for the dark, rectangular, G5-inspired SVG instrument selected for #4.
* Render the latest accepted sample on the next animation frame without interpolation.
* Reconsider smoothing only after observing mobile behavior.
* Separate source and transport status.
* Obscure unavailable attitude instead of displaying a credible level-flight indication.
* Use [PFD scope](pfd-demo-scope.md) for extensions beyond the initial attitude-only demo.
* Reuse the [recorded phone environments](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/4#issuecomment-6065793741) when later feedback explicitly states that the same devices were used.

| Evidence | Limit |
| --- | --- |
| Desktop viewports / synthetic SSE | Browser behavior only, not phone lifecycle or simulator compatibility |
| Real iOS Safari / Android Chrome | Run the documented procedures on actual devices. Keep the feature issue open until required device evidence exists |

Sources: accepted decisions for [#4](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/4) and [#19](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/19), 2026-10-08.
Procedures: [testing](../docs/testing.md). Freshness/status: [contract](../docs/telemetry-contract.md).

* For #37 and later, fill the entire panel with sky/ground and retain a separate central pitch clip.
* Keep the top status bar removed; show small DEMO in the lower left only for demo data and unavailable reasons on the full-panel cover.
* Use the development fixture to compare background opacity candidates; `0.65` is provisional until real-phone acceptance selects a value.

Source: approved #37 implementation plan and explicit full-panel/background and source-badge choices, 2026-10-10; implementation details belong in [panel presentation](../docs/pfd-presentation.md).
