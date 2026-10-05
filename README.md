# MSFS2024 Artificial Horizon

A mobile web attitude display for Microsoft Flight Simulator 2024, inspired by
the readability of the Garmin G5. Rust runs the HTTP server; a lightweight
HTML/CSS/JavaScript client receives telemetry through Server-Sent Events (SSE).

The Rust library, CLI, and server target Windows x64 MSVC only, with Windows-only
CI. The web client supports iOS Safari and Android Chrome.

**Status:** project preparation. Architecture, delivery plan, and GitHub tasks
are ready; the application is not implemented or runnable yet.

## Intended experience

1. Start the application on the Windows PC running MSFS2024.
2. Connect a phone to the same local network.
3. Open the server address, for example `http://192.168.1.20:8080`.
4. The page automatically opens a read-only telemetry stream and displays attitude.

The address above is illustrative. No cloud service, account, phone application,
or browser-to-simulator controls are planned for the MVP. A browser session means
an active SSE subscription, not a persisted login session.

## MVP

- Pitch and bank, with a horizon, pitch ladder, bank scale, and fixed aircraft reference.
- Responsive portrait and landscape display with explicit connection/data status.
- An explicitly selected demo mode for development without the simulator.
- A Windows SimConnect provider, subject to a compatibility spike.
- Automatic browser reconnection and clear stale/paused/disconnected indication.

Airspeed/altitude tapes, heading, navigation, flight director, autopilot controls,
recording, internet hosting, and a full G5 replica are outside the initial scope.
This is a simulator companion, not a real-flight instrument.

## Project guide

| Document | Purpose |
| --- | --- |
| [Project brief](docs/project-brief.md) | Requirements, assumptions, scope, and success criteria |
| [Architecture](docs/architecture.md) | Components, intended source layout, and deployment |
| [Architecture decision](docs/adr/0001-rust-http-sse.md) | Stack choice and trade-offs |
| [Telemetry contract](docs/telemetry-contract.md) | HTTP/SSE interface, units, and freshness |
| [Roadmap](docs/roadmap.md) | Delivery order and linked GitHub Issues |
| [Testing](docs/testing.md) | Automated checks and real-device acceptance |
| [Contributing](CONTRIBUTING.md) | Issue and branch workflow |

Work is tracked in [GitHub Issues](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues).
Code, documentation, issues, commit messages, and pull requests use English.

## Implementation starting point

Start with [#1: Rust skeleton and CI](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/1).
Initialize a Rust library package with a thin `src/bin/main.rs` entry point and
clap CLI, so future applications can share the library. HTTP liveness uses `/health`.
Then build the demo-to-browser path while the SimConnect investigation resolves
SDK integration choices. Installation and run commands will be added when they
have been tested against an actual application.

## License

[MIT](LICENSE).
