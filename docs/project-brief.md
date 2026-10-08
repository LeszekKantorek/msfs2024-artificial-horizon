# Project brief

## Goal

Let a Microsoft Flight Simulator 2024 user open a web address on a phone for a
readable flight display. Extend the attitude demo into a responsive PFD while
keeping the existing live-simulator scope limited to pitch and bank. Prioritize
mobile legibility, low delay, and honest indication when data is unavailable.

## Requirements

- Rust server with HTTP and server-to-browser telemetry delivery over SSE.
- Windows x64 MSVC is the only supported platform for the Rust library, CLI,
  server, and CI. The web client runs in mobile browsers.
- Rust library package with reusable application logic and executable entry points
  under `src/bin/`; clap for command-line argument parsing.
- HTML, CSS, JavaScript, and SVG client with no application framework/build pipeline.
- Mobile browser support: iOS Safari and Android Chrome, portrait and landscape.
- Automatic subscription on page load and recovery after a connection interruption.
- Read-only simulator integration: no control commands from the browser.
- G5-inspired instrument hierarchy and legibility, using original visual assets.
- GitHub Issues for implementation work; English for all repository content.

## Working assumptions

The server runs on the Windows x64 PC running MSFS2024. Phones share its trusted
local network. Internet access and remote hosting are not needed during use.
These are planning assumptions, not deployment behavior already implemented.
If remote access is required, revisit deployment and access control before adding it.

One shared aircraft source feeds every connected phone. There are no accounts,
database, per-user simulator sessions, or persistent telemetry history.

## Scope and acceptance

The original simulator MVP covers pitch and bank, scales, an aircraft reference,
and source/connection status. Issues #5-#8 retain their existing scope and
dependencies. A simulator failure must never silently select demo data.

The accepted PFD expansion is planned demo functionality, delivered in separate
feature slices. All additional values, including selected altitude, selected
heading/track and barometric setting, come from deterministic Rust demo scenarios.
The browser remains read-only, without a menu, knob, editing or sync actions.
Connecting these additional indications to MSFS2024 is separate future work;
the attitude-only simulator provider must leave them unavailable.

### PFD reference and coverage

Use the owner-supplied Garmin G5 Part 23 AML STC Pilot's Guide,
`190-01112-12_02.pdf`, document 190-01112-12 Rev. 2, section 3.3. The illustration
labelled **Page 8 of 25** is physical PDF page **13**. Subsequent subsections define
instrument behavior. The PDF is a reference, not an application asset; use
original graphics rather than embedding its screenshots or copying the bezel.

| Included illustration items | PFD capability | Feature issue |
| --- | --- | --- |
| 2, 3, 5, 7, 10 | Attitude, pitch scale, aircraft reference, slip/skid and turn rate | [#20](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/20) |
| 1, 4, 8 | Airspeed tape, current IAS and ground speed | [#21](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/21) |
| 21; extends 1 | V-speed references, speed ranges, six-second trend and VNE indications | [#22](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/22) |
| 9, 13, 15 | Barometric setting, current altitude and altimeter tape | [#23](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/23) |
| 11, 16 | Selected-altitude bug, readout and approach/deviation alerts | [#24](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/24) |
| 12 | Vertical speed | [#25](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/25) |
| 18, 19, 20 | Heading/track tape, current track and selected direction | [#26](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/26) |

This covers 18 of the 22 illustration items. Exclude CDI (6), vertical navigation
guidance (14), navigation course (17) and battery (22). No HSI screen, ILS,
GPS glidepath or VNAV is included. Do not reserve UI space, model fields or demo
scenarios for excluded functions. Heading/ground track remain PFD indications.
Flight director, autopilot controls, recording and internet hosting remain out
of scope. Fullscreen/wake lock and PWA installation are later candidates.

Initial units are kt, ft, ft/min, inHg and degrees. Speed thresholds belong to an
explicit synthetic demo profile, not assumed performance of a real aircraft.

### Mobile presentation

The planned PFD has **no fixed aspect ratio**, including no 4:3 requirement.
Adapt its geometry to available width and height rather than uniformly scaling
or stretching a complete instrument image.

- Portrait: attitude in the center, IAS on the left, altitude/VSI on the right,
  heading/track above, and supplemental values near their related instruments.
- Landscape: use the extra width and reduced height while retaining familiar
  relative positions of the primary instruments.
- Size digits, symbols, tapes and horizon independently; preserve undistorted
  symbols and enough central space for readable attitude.
- Minimize bezel decoration, headings and margins; keep DEMO and separate
  source/transport status compact, visible and unambiguous.
- Account for safe areas, screen cutouts and browser bars; fit the primary panel
  without scrolling from 320 CSS px in both orientations.
- In #20, assess the complete intended arrangement using development-only layout
  fixtures, so later slices do not squeeze the horizon or require a redesign.

### Acceptance ownership

Each PFD issue delivers the typed data, shared deterministic demo, SSE consumer,
visible indication, focused tests and affected documentation. Mobile readability
is an acceptance condition from the first slice onward. Record real iOS Safari
and Android Chrome evidence in those issues; #26 also owns accumulated full-panel
acceptance of all 18 included elements. This does not expand #7 or establish live
PFD compatibility.

Missing data must not become zero. An invalid new instrument value affects that
indication; source/transport loss invalidates the panel. Preserve reconnect,
fresh-data-on-resume and sample-expiry behavior through every slice.

Proposed performance targets, to validate on real equipment:

| Measure | Initial target |
| --- | --- |
| Sample publication | 20 Hz, configurable after measurement |
| End-to-end latency on healthy LAN | p95 <= 150 ms |
| Active-page stale-data indication | Within 1 second of the last usable sample |
| Layout | Usable from 320 CSS pixels wide, both orientations |
| Concurrent use | Two phones for 30 minutes without growing queues |

Success requires visible correct behavior for level flight, nose up/down, left/right
bank, and combined attitudes; recovery after network/server interruption; and
explicit paused, disconnected, invalid, and stale states. A steady aircraft is not
stale merely because its attitude values remain unchanged.

## Decisions still requiring evidence

- Rust SimConnect wrapper versus isolated FFI, including SDK/runtime distribution.
- Actual simulator sign/unit conversion and lifecycle event behavior.
- Mobile performance and whether interpolation is necessary.
- The first tested aircraft, phones, and browser versions; record them in validation.

See the [roadmap](roadmap.md) for ownership by task and dependencies.
