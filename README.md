# MSFS2024 Artificial Horizon

A compact electronic flight instrument (EFI) for your phone, centered on an artificial horizon.
Start the Rust server on Windows,
then open its address in a browser on the same network.

**Windows x64** · **Read-only browser display** · **No phone app or account**

> **Try the demo today.** The executable displays synthetic attitude, IAS, TAS, GS, altitude, selected-altitude alerts, vertical speed, pressure, magnetic heading, slip/skid, and turn rate.
> SimConnect feeds available live flight instruments to the same browser display. Speed ranges and references remain independently available; unsupported aircraft data is hidden.
> See the [recorded evidence and limits](docs/sources/simconnect.md#live-publication-evidence). This is not a real-flight instrument.

[Start the demo](#start-the-demo) · [Use your phone](#use-your-phone) · [Development](CONTRIBUTING.md)

## Start the demo

You need:

* Windows x64, Git, and Rust stable through rustup, using the MSVC toolchain.
* Visual Studio C++ Build Tools with the MSVC linker and Windows SDK.

The demo requires no MSFS installation, SimConnect SDK, Node.js, or frontend build.

### 1. Get the source

Run in PowerShell:

```powershell
git clone https://github.com/LeszekKantorek/msfs2024-artificial-horizon.git
cd msfs2024-artificial-horizon
```

### 2. Start the server

```powershell
rustup default stable
$env:CARGO_BUILD_TARGET = 'x86_64-pc-windows-msvc'
cargo build --workspace --locked
Copy-Item default.json target/x86_64-pc-windows-msvc/debug/default.json
cargo run --locked --bin msfs2024-artificial-horizon -- --scenario demo-scenarios/demo-scenario.jsonl
```

The workspace build places `demo.dll` beside the executable. The copy command places the mandatory mapping there too.

### 3. Open the display

Check the display and stop the server when finished:

1. Open [http://127.0.0.1:8080](http://127.0.0.1:8080) on the PC.
2. Check that attitude moves and that the left airspeed tape has TAS above it and ground speed.
   The right altitude tape must show the pressure setting.
3. Press `Ctrl+C` in PowerShell to stop the server gracefully.

For automated shutdown, send `stop` followed by a newline to standard input.
This requests the same coordinated shutdown as `Ctrl+C`.

## Use your phone

Connect the phone to the server through the trusted private network:

1. Connect the PC and phone to the same trusted private network.
2. Stop the local server with `Ctrl+C` if it is still running.
3. Start the server with LAN access:

   ```powershell
   cargo run --locked --bin msfs2024-artificial-horizon -- --source demo.dll --scenario demo-scenarios/demo-scenario.jsonl --listen-address 0.0.0.0 --port 8080
   ```

4. Allow inbound TCP port `8080` in Windows Defender Firewall for the Private profile and local subnet.
5. Open the printed LAN URL on the phone.

```text
Windows PC                         Phone browser
Rust demo + HTTP server  --Wi-Fi--> Safari / Chrome
                         telemetry
```

Apply these browser and connection requirements:

* Use iOS Safari or Android Chrome. Rotate between portrait and landscape.
* Text selection is disabled on the instrument panel to prevent accidental selection when touching the display.
* Check the printed interface when a VPN or multiple adapters are active.
* If address detection fails, use `ipconfig` to find the PC's private IP.
* You can bind that IP directly with `--listen-address`.

Apply these LAN access constraints:

* Use the PC's LAN IP on the phone. `localhost` refers to the phone itself.
* Do not forward the server port on your router. LAN access has no authentication.

## Options and troubleshooting

```powershell
cargo run --locked --bin msfs2024-artificial-horizon -- --help
cargo run --locked --bin msfs2024-artificial-horizon -- --version
cargo run --locked --bin msfs2024-artificial-horizon -- --source demo.dll --scenario demo-scenarios/demo-scenario.jsonl --listen-address 127.0.0.1 --port 9000
```

| Situation | Action / constraint |
| --- | --- |
| Port occupied | Select another `--port` from `1` to `65535` |
| Invalid address | Use an IPv4/IPv6 address, not a hostname, multicast address, or IPv4 broadcast address |
| Phone cannot connect | Check the private network, PC address, port, VPN interface, and firewall scope |
| SimConnect DLL cannot load | Place the official x64 DLL beside the executable or supply an absolute `--source` path. Demo never replaces SimConnect automatically |
| Mapping cannot load | Place the validated `default.json` beside the executable, then restart |

Select native reception with `--source SimConnect.dll`:

```powershell
cargo build --workspace --locked --release
Copy-Item default.json target/x86_64-pc-windows-msvc/release/default.json
cargo run --locked --release --bin msfs2024-artificial-horizon -- --source SimConnect.dll --log ".local\simconnect-sessions\active-pause.jsonl"
```

Configure native reception under these requirements:

* Place `SimConnect.dll` beside the running executable, or pass its absolute path to `--source`.
  The default DLL location is independent of the working directory.
* If using `--log`, supply the exact file to append. Use distinct filenames to distinguish scenarios.
* Follow [SimConnect setup and scenarios](docs/sources/simconnect.md) for the applicable requirements and observation procedures.
* Use the same HTTP/SSE address and port options as Demo.
  Add `--listen-address 0.0.0.0 --port 8080` for access from the trusted private network.
* Open the printed URL on the PC or phone.

Interpret source availability and recorded results under these limits:

* If SimConnect cannot start, HTTP remains available with unavailable display data. The console explains the source failure.
* Live publication requires a running simulator, a loaded flight, zero pause flags, and a fresh valid sample.
* Recorded sessions support signs, pause/menu unavailability, fresh-sample recovery, and reconnect in the tested setup.
  The [evidence and limits](docs/sources/simconnect.md#live-publication-evidence) define that coverage.
* The #5 receiver evidence does not establish integrated live-browser or mobile acceptance.
  Broader compatibility and complete performance coverage remain unverified.

## Project guide

| I want to… | Read |
| --- | --- |
| Run checks or test a phone | [Testing](docs/testing/index.md) |
| Contribute a change | [Contributing](CONTRIBUTING.md) |
| Understand planning and execution | [Work organization](docs/work-organization.md) |
| Use the Rust library or understand the design | [Architecture](docs/architecture.md#application-composition) |
| Understand HTTP APIs, assets, and connection shutdown | [Server](docs/server.md) |
| Develop or validate a source provider | [Sources](docs/sources/index.md) |
| Find deterministic Demo scenarios | [Demo](docs/sources/demo.md) |
| Consume telemetry | [Telemetry contract](docs/telemetry-contract.md) |
| Run SimConnect reception or diagnostics | [SimConnect setup and scenarios](docs/sources/simconnect.md) |
| Review scope and future work | [EFI scope](docs/efi/index.md#product-scope-and-reference) · [MVP Epic target #9](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/9), with [migration boundaries](docs/work-organization.md#adoption-boundaries) |
| Understand the stack choice | [ADR 0001](docs/adr/0001-rust-http-sse.md) |
| Track work | [GitHub Issues](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues) |

[MIT license](LICENSE).
