# Project brief

## Goal

Let a Microsoft Flight Simulator 2024 user use a phone as a dedicated artificial
horizon by opening a web address. Prioritize readable attitude, low delay, and
honest indication when data is unavailable.

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

The MVP shows pitch and bank, scales, an aircraft reference, and source/connection
status. It includes demo and simulator sources; a simulator failure must never
silently select demo data. Each source uses the same data contract.

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

## Later candidates

Airspeed, altitude, vertical speed, heading, optional fullscreen/wake lock, and
installation as a PWA can become separate issues after the basic instrument is
validated. They are not acceptance conditions for the first release.

## Decisions still requiring evidence

- Rust SimConnect wrapper versus isolated FFI, including SDK/runtime distribution.
- Actual simulator sign/unit conversion and lifecycle event behavior.
- Mobile performance and whether interpolation is necessary.
- The first tested aircraft, phones, and browser versions; record them in validation.

See the [roadmap](roadmap.md) for ownership by task and dependencies.
