# Project brief

## Goal

Display MSFS2024 flight data on a phone through a web address.
Prioritize mobile readability, low delay, and explicit unavailable states.

> Extend the demo into a responsive primary flight display (PFD). Keep live-simulator scope limited to pitch and bank.

## Requirements

| Area | Requirement |
| --- | --- |
| Server | Rust, HTTP, server-to-browser Server-Sent Events (SSE) |
| Platform | Windows x64 MSVC for library, CLI, server, and CI |
| Package | Reusable library, executables under `src/bin/`, clap CLI |
| Client | HTML, CSS, JavaScript, SVG. No application framework or build pipeline |
| Browsers | iOS Safari and Android Chrome, portrait and landscape |
| Connection | Subscribe on page load. Recover automatically after interruption |
| Simulator access | Read-only. No browser control commands |
| Visuals | G5-inspired hierarchy and readability, original assets |
| Project | GitHub Issues track work. All repository content uses English |

## Working assumptions

* The server runs on the Windows x64 PC that runs MSFS2024.
* Phones share the PC's trusted local network.
* Use requires no internet access or remote hosting.
* One aircraft source feeds all phones.
* There are no accounts, database, per-user simulator sessions, or persistent telemetry history.

> These are planning assumptions, not claims of implemented deployment behavior. Revisit deployment and access control before adding remote access.

## Scope and acceptance

| Scope | Data and controls | Boundary |
| --- | --- | --- |
| Original simulator MVP | Pitch/bank, scales, aircraft reference, source/connection status | Preserve #5-#8 scope and dependencies |
| Planned PFD demo | Deterministic Rust scenarios, including selected altitude, heading/track, and barometric setting | Deliver separate feature slices |
| Future live PFD | Additional MSFS2024 indications | Requires separate scope. The attitude-only provider leaves these unavailable |
| Browser controls | Read-only presentation | No menu, knob, editing, or sync actions |

> Simulator failure must never select demo data silently.

### PFD reference and coverage

| Reference | Location |
| --- | --- |
| Reference guide | Garmin G5 Part 23 AML STC Pilot's Guide |
| File / revision | `190-01112-12_02.pdf`, document 190-01112-12 Rev. 2 |
| Illustration | Section 3.3, printed **Page 8 of 25**, physical PDF page **13** |
| Behavior | Subsequent subsections |

Use original graphics. Do not embed PDF screenshots or copy the bezel.

| Included illustration items | PFD capability | Feature issue |
| --- | --- | --- |
| 2, 3, 5, 7, 10 | Attitude, pitch scale, aircraft reference, slip/skid, turn rate | [#20](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/20) |
| 1, 4, 8 | Airspeed tape, current IAS, ground speed | [#21](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/21) |
| 21, extends 1 | V-speed references, speed ranges, six-second trend, VNE indications | [#22](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/22) |
| 9, 13, 15 | Barometric setting, current altitude, altimeter tape | [#23](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/23) |
| 11, 16 | Selected-altitude bug/readout, approach/deviation alerts | [#24](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/24) |
| 12 | Vertical speed | [#25](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/25) |
| 18, 19, 20 | Heading/track tape, current track, selected direction | [#26](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/26) |

**Coverage: 18 of 22 illustration items.**

| Exclusion | Rule |
| --- | --- |
| CDI (6), vertical navigation guidance (14), navigation course (17), battery (22) | No UI space, model fields, or demo scenarios |
| HSI, ILS, GPS glidepath, VNAV | Outside scope. Heading/ground track remain PFD indications |
| Flight director, autopilot controls, recording, internet hosting | Outside scope |
| Full G5 replica | Outside scope |
| Fullscreen, wake lock, PWA installation | Later candidates |

* Initial units: kt, ft, ft/min, inHg, degrees.
* Speed thresholds belong to an explicit synthetic profile, not assumed real-aircraft performance.

### Mobile presentation

> No fixed aspect ratio, including 4:3. Adapt geometry to available width and height.

| Area | Layout rule |
| --- | --- |
| Portrait | Attitude center, IAS left, altitude/VSI right, heading/track above, supplemental values near related instruments |
| Landscape | Use extra width and reduced height. Retain familiar instrument positions |
| Instrument sizing | Size digits, symbols, tapes, and horizon independently. Preserve undistorted symbols and readable central attitude |
| Page chrome | Minimize bezel, headings, and margins |
| Status | Keep DEMO and separate source/transport status compact, visible, and unambiguous |
| Viewport | Account for safe areas, cutouts, and browser bars. Fit without scrolling from 320 CSS px in both orientations |
| Minimum usable height | 240 CSS px after browser bars and safe areas, including short landscape |
| First slice (#20) | Assess all intended elements with development-only layout fixtures before later additions reduce horizon space |

### Planned layered composition

The opaque sky/ground background extends behind the side instruments.
Tape backgrounds are translucent; digits, ticks, and value windows remain readable.
Central pitch markings have separate clipping, and invalid data remains obscured.
The [panel presentation](pfd-presentation.md#layers-and-clipping) defines layer ownership and clipping.

[#36](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/36) first separates existing modules without changing the image.
[#37](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/37) then implements the composition and selects opacity through mobile acceptance.
These steps add no telemetry fields or functional tape readings.

### Acceptance ownership

Each instrument feature issue owns:

* Typed data, shared deterministic demo, SSE consumer, and visible indication.
* Focused tests and affected documentation.
* Readability evidence from real iOS Safari and Android Chrome, starting with the first slice.

#26 also owns full-panel acceptance of all 18 elements.
This does not expand #7 or establish live PFD compatibility.

Acceptance criteria:

* [ ] Missing data never becomes zero.
* [ ] An invalid optional instrument value affects only that indication.
* [ ] Source/transport loss invalidates the panel.
* [ ] Reconnect, fresh data after resume, and sample expiry remain correct through every slice.
* [ ] Level, nose up/down, left/right bank, and combined attitudes display correctly.
* [ ] Network/server interruptions recover automatically.
* [ ] Paused, disconnected, invalid, and stale states remain explicit.
* [ ] Unchanged attitude with fresh callbacks remains valid.

Proposed targets require measurements on real equipment:

| Measure | Initial target |
| --- | --- |
| Sample publication | 20 Hz, configurable after measurement |
| End-to-end latency, healthy LAN | p95 <= 150 ms |
| Active-page stale indication | Within 1 second of the last usable sample |
| Layout | Usable from 320 CSS px, both orientations |
| Concurrent use | Two phones for 30 minutes without growing queues |

> Checklists define acceptance criteria, not completion status. Record results in the owning issues.

## Decisions still requiring evidence

* Rust SimConnect wrapper versus isolated FFI, including SDK/runtime distribution.
* Simulator sign/unit conversion and lifecycle behavior.
* Mobile performance and the need for interpolation.
* First tested aircraft, phones, and browser versions.

Record tested versions in validation. See the [roadmap](roadmap.md) for issue ownership and dependencies.
