# ADR 0004: Compose the PFD from instrument modules

| Date | Status |
| --- | --- |
| 2026-10-10 | Accepted |

## Context

The attitude implementation combines several instruments in one SVG view module.
Adding tapes to that module would make geometry and update responsibilities harder to follow.
The target composition places side instruments over a shared attitude background.
[ADR 0003](0003-responsive-pfd-demo.md) already separates layout, geometry, and DOM updates.

## Decision

* Keep plain JavaScript, SVG, SSE, and latest-sample rendering without interpolation.
* Compose the panel from instrument modules with local coordinates and pure geometry calculations.
* Use `panel.resize(layout)`, `panel.render(frame)`, and `panel.invalidate(reason)` as the presentation interface.
* Keep frame scheduling independent of attitude mathematics.
* Preserve one sample deadline through layout changes and cancel pending drawing on invalidation.
* Put the opaque attitude background behind translucent tape backgrounds.
* Keep text, ticks, and value windows readable, with separate clipping for central pitch markings.
* Deliver the structural refactor before changing the composition.
* Add future instruments only in their existing feature issues.

The [presentation document](../pfd-presentation.md) owns detailed responsibilities, interfaces, and layer rules.
The [roadmap](../roadmap.md) owns delivery order and technical prerequisites.
Planning is tracked in [#35](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/35).
Implementation is tracked in [#36](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/36) and [#37](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/37).

## Alternatives and consequences

| Alternative | Decision |
| --- | --- |
| Continue extending one view module | Reject because unrelated instrument geometry remains coupled |
| Instrument modules with one frame scheduler | Select for explicit ownership and a traceable update path |
| Solid.js, three.js, or a custom signals system | Defer because module boundaries do not require a technology migration |

Two preparation steps precede the remaining feature slices.
The refactor preserves the existing image; the next step changes background coverage and layering.
Layer validation must distinguish intentional overlap from content collisions.
Both steps require regression evidence, including real-phone checks.
No wire contract or live-simulator scope changes follow from this decision.

> Acceptance records the design decision, not completed implementation or device validation.
