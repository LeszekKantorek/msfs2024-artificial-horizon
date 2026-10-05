# Delivery roadmap

Track progress in [MVP issue #9](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/9).
GitHub Issues own task status and acceptance criteria. This document defines order
and dependencies; it deliberately does not duplicate open/closed status.

## Stage 1: Demo from source to phone

| Order | Issue | Depends on | Deliverable |
| --- | --- | --- | --- |
| 1 | [#1 Rust skeleton and CI](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/1) | None | Buildable server, static page, health, checks |
| 2 | [#2 Telemetry and demo provider](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/2) | #1 | Typed current state and repeatable attitudes |
| 3 | [#3 SSE sessions](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/3) | #1, #2 | Current snapshots, reconnect, bounded consumers |
| 4 | [#4 Mobile instrument](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/4) | #2, #3 | Responsive attitude and honest status |

Exit: a phone displays explicitly marked demo attitude and recovers from a dropped
connection without manual reload. This establishes the full presentation path
without depending on the simulator SDK.

## Stage 2: Real simulator data

| Order | Issue | Depends on | Deliverable |
| --- | --- | --- | --- |
| Early investigation | [#5 SimConnect compatibility spike](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/5) | None | Proven binding/setup, signs, units, lifecycle |
| Integration | [#6 Windows SimConnect provider](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/6) | #2, #5 | Live data, reconnect, pause/flight lifecycle |

Start #5 early; it can proceed independently of Stage 1. Its initial one-day
timebox produces evidence or a concrete blocker, not an assumption of success.
SDK/binding decisions are deferred until that evidence exists.

Exit: the same instrument consumes real MSFS2024 attitude and represents unavailable
or paused simulation explicitly. Demo mode remains separately selectable.

## Stage 3: Validate and distribute

| Order | Issue | Depends on | Deliverable |
| --- | --- | --- | --- |
| Validation | [#7 Mobile reliability and latency](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/7) | #4, #6 | Device evidence and measured targets |
| Packaging | [#8 Windows MVP release](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/8) | #6, #7 | Executable, prerequisites, tested setup guide |

Exit: a user launches the Windows package and connects a phone using documented
steps, without development tools. The release notes distinguish measured behavior
from remaining limitations.

## Change control

Create a new issue for additional instruments or deployment modes. Change scope
and acceptance criteria explicitly before adding them to the MVP. Close issues
only when evidence meets their criteria; repository preparation does not close
the implementation tasks above.
