# MSFS2024 Artificial Horizon

A mobile web attitude display for Microsoft Flight Simulator 2024, inspired by
the readability of the Garmin G5. Rust runs the HTTP server; a lightweight
HTML/CSS/JavaScript client receives telemetry through Server-Sent Events (SSE).

The Rust library, CLI, and server target Windows x64 MSVC only, with Windows-only
CI. The web client supports iOS Safari and Android Chrome.

**Status:** HTTP application skeleton. The embedded placeholder page and `/health`
are available. Telemetry, demo trajectories, SSE, and the instrument are planned
in issues #2-#4; SimConnect integration is planned in #5-#6.

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

## Build and run

Use Windows x64 with rustup and the Visual Studio C++ Build Tools (MSVC linker
and Windows SDK). No MSFS installation or SimConnect SDK is needed. The recorded
Rust toolchain is 1.98.1; rustup reads `rust-toolchain.toml` automatically.

```powershell
cargo build --locked
cargo run --locked --bin main
```

Open `http://127.0.0.1:8080`. The default source is `demo`, but this skeleton
produces no telemetry. `GET /health` returns `{"status":"ok"}` for HTTP liveness,
not simulator readiness. Press Ctrl+C for graceful shutdown.

```powershell
cargo run --locked --bin main -- --help
cargo run --locked --bin main -- --version
cargo run --locked --bin main -- --source demo --listen-address 127.0.0.1 --port 9000
```

`--listen-address` accepts an IPv4 or IPv6 address, not a hostname. `--port`
accepts 1-65535. Multicast and IPv4 broadcast addresses are rejected. An occupied
port produces an error; choose another port. Selecting `--source simconnect`
fails explicitly until integration is implemented, even with `--features simconnect`.
The feature is currently an SDK-free Windows module boundary.

## Explicit LAN access

```powershell
cargo run --locked --bin main -- --source demo --listen-address 0.0.0.0 --port 8080
```

The startup output reports a detected LAN URL. Verify the interface when using
a VPN or multiple adapters; alternatively bind your PC's private IP directly.
If detection fails, find the private IP with `ipconfig` and use that IP in the
phone URL. `localhost` on the phone refers to the phone itself.

Allow inbound TCP on the chosen port in Windows Defender Firewall, scoped to
the Private profile and local subnet. Use the same trusted private network;
do not forward the port on your router. LAN access has no authentication.
This skeleton exposes only its placeholder and liveness, not flight data.

## Library and checks

The `msfs2024_artificial_horizon` library exposes `Config`, `Source`, and `Server`.
`Server::bind(config).await` validates and binds; `local_address()` returns the
bound address; `run(shutdown).await` takes a caller-owned shutdown future.
The library never reads process arguments or installs a Ctrl+C handler.

```powershell
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo clippy --locked --all-targets --features simconnect -- -D warnings
cargo test --locked --all-targets --features simconnect
```

CI runs the same checks on Windows x64 MSVC. See [testing](docs/testing.md)
for the skeleton acceptance procedure and later simulator/mobile validation.

## License

[MIT](LICENSE).
