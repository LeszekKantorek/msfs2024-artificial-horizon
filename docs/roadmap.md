# Delivery roadmap

Track progress in [MVP issue #9](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/9).
GitHub Issues own task status and acceptance criteria. This document defines order
and dependencies; it deliberately does not duplicate open/closed status.

## Stage 1: Demo from source to phone

| Order | Issue | Depends on | Deliverable |
| --- | --- | --- | --- |
| 1 | [#1 Rust skeleton and CI](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/1) | None | Library, thin clap CLI, static page, `/health`, checks |
| 2 | [#2 Telemetry and demo provider](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/2) | #1 | Typed current state and repeatable attitudes |
| 3 | [#3 SSE sessions](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/3) | #1, #2 | Current snapshots, reconnect, bounded consumers |
| 4 | [#4 Mobile instrument](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/4) | #2, #3 | Responsive attitude and honest status |

Exit: a phone displays explicitly marked demo attitude and recovers from a dropped
connection without manual reload. This establishes the full presentation path
without depending on the simulator SDK.

## Stage 2: G5 PFD demo vertical slice

The [project brief](project-brief.md) defines the 18 included reference elements,
exclusions and responsive layout without a fixed aspect ratio. Scope/backlog work
belongs to [#19](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/19).

| Order | Issue | Depends on | Deliverable |
| --- | --- | --- | --- |
| G1 | [#20 Responsive PFD attitude and coordinated turn](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/20) | #4 | Mobile layout, refined attitude, slip/skid and turn rate |
| G2 | [#21 Airspeed and ground speed](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/21) | #20 | IAS tape/readout and GS |
| G3 | [#22 Speed ranges, V-speeds and trend](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/22) | #21 | Demo profile, references, trend and VNE indications |
| G4 | [#23 Altitude and barometric setting](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/23) | #20 | Altitude tape/readout and baro |
| G5 | [#24 Selected altitude and alerts](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/24) | #23 | Target bug/readout and approach/deviation alerts |
| G6 | [#25 Vertical speed](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/25) | #23 | VSI scale and indication |
| G7 | [#26 Heading, track and selected direction](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/26) | #20 | PFD direction tape and accumulated full-panel demo acceptance |

Deliver in G1-G7 order; the dependency column records technical prerequisites,
not extra dependencies on all preceding rows. Each issue owns one complete slice
from typed data and deterministic Rust demo through SSE to presentation, tests
and documentation. Use one reviewable feature PR per issue. Selected values are
read-only demo data; there is no battery, HSI or navigation-guidance slice.

Exit: all 18 included elements meet their criteria on actual iOS Safari and
Android Chrome, with readable portrait/landscape layouts from 320 CSS px, explicit
unavailable indications and recovery without reload. Each slice owns its mobile
evidence; #26 collects full-panel acceptance. This establishes demo behavior only.

The following two stages retain their existing issue scope and dependencies.
New PFD values remain unavailable in the attitude-only simulator provider; live
integration of those values requires a separate future scope decision.

## Stage 3: Real simulator data

| Order | Issue | Depends on | Deliverable |
| --- | --- | --- | --- |
| Early investigation | [#5 SimConnect compatibility spike](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/5) | None | Proven binding/setup, signs, units, lifecycle |
| Integration | [#6 Windows SimConnect provider](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/6) | #2, #5 | Live data, reconnect, pause/flight lifecycle |

Start #5 early; it can proceed independently of both demo stages. Its initial one-day
timebox produces evidence or a concrete blocker, not an assumption of success.
SDK/binding decisions are deferred until that evidence exists.

Exit: the same instrument consumes real MSFS2024 attitude and represents unavailable
or paused simulation explicitly. Demo mode remains separately selectable.

## Stage 4: Validate and distribute

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
