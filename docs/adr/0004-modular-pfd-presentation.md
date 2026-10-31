# ADR 0004: Compose the PFD from instrument modules

Decision status: Accepted.

## Context

Combining unrelated instruments in one SVG module obscures geometry and update responsibilities.
Instrument composition places side instruments over a shared attitude background.
[ADR 0003](0003-responsive-pfd-demo.md) separates layout, geometry, and DOM updates.

## Decision

* Keep plain JavaScript, SVG, SSE, and latest-sample rendering without interpolation.
* Compose the panel from instrument modules with local coordinates and pure geometry calculations.
* Use `panel.resize(layout)`, `panel.render(frame)`, and `panel.invalidate(reason)` as the presentation interface.
* Pass the same complete, read-only frame to every instrument render method.
* Let each instrument select its data without changing panel delegation when its data needs expand.
* Keep frame scheduling independent of attitude mathematics.
* Preserve one sample deadline through layout changes and cancel pending drawing on invalidation.
* Put the opaque attitude background behind translucent tape backgrounds.
* Keep text, ticks, and value windows readable, with separate clipping for central pitch markings.
* Validate structural refactoring separately from intentional composition changes.
* Add instruments only within the accepted product scope.

The [presentation document](../efi/index.md) owns detailed responsibilities, interfaces, and layer rules.
The [roadmap](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/9) owns delivery order and technical prerequisites.

## Alternatives and consequences

| Alternative | Decision |
| --- | --- |
| Continue extending one view module | Reject because unrelated instrument geometry remains coupled |
| Instrument modules with one frame scheduler | Select for explicit ownership and a traceable update path |
| Solid.js, three.js, or a custom signals system | Defer because module boundaries do not require a technology migration |

Composition changes require these checks:

* Preserve appearance during structural refactoring unless a visual change is explicitly accepted.
* Validate intentional changes to background coverage and layering separately.
* Distinguish intentional overlap from content collisions.
* Record regression evidence, including real-phone checks, for refactoring and composition changes.
* Keep wire contracts and live-simulator scope independent of visual composition decisions.

> Acceptance records the design decision, not completed implementation or device validation.
