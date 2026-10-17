# MSFS2024 Artificial Horizon

An aircraft attitude display for your phone. Start the Rust server on Windows,
then open its address in a browser on the same network.

**Windows x64** · **Read-only browser display** · **No phone app or account**

> **Try the demo today.** The executable displays synthetic attitude, IAS, TAS, GS, altitude, selected-altitude alerts, vertical speed, pressure, magnetic heading, slip/skid, and turn rate.
> MSFS2024 integration remains planned. This is not a real-flight instrument.

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
cargo run --locked --bin msfs2024-artificial-horizon
```

Cargo builds the application before starting it.

### 3. Open the display

Open [http://127.0.0.1:8080](http://127.0.0.1:8080) on the PC.
Expect a moving attitude display, a left-side airspeed tape with TAS above it, ground speed, and a right-side altitude tape with pressure setting.
Press `Ctrl+C` in PowerShell to stop the server gracefully.

## Use your phone

1. Connect the PC and phone to the same trusted private network.
2. Stop the local server with `Ctrl+C` if it is still running.
3. Start the server with LAN access:

   ```powershell
   cargo run --locked --bin msfs2024-artificial-horizon -- --source demo --listen-address 0.0.0.0 --port 8080
   ```

4. Allow inbound TCP port `8080` in Windows Defender Firewall for the Private profile and local subnet.
5. Open the printed LAN URL on the phone.

```text
Windows PC                         Phone browser
Rust demo + HTTP server  --Wi-Fi--> Safari / Chrome
                         telemetry
```

* Use iOS Safari or Android Chrome. Rotate between portrait and landscape.
* Check the printed interface when a VPN or multiple adapters are active.
* If address detection fails, use `ipconfig` to find the PC's private IP.
* You can bind that IP directly with `--listen-address`.

> Use the PC's LAN IP on the phone. `localhost` refers to the phone itself.
> LAN access has no authentication. Do not forward the port on your router.

## Options and troubleshooting

```powershell
cargo run --locked --bin msfs2024-artificial-horizon -- --help
cargo run --locked --bin msfs2024-artificial-horizon -- --version
cargo run --locked --bin msfs2024-artificial-horizon -- --source demo --listen-address 127.0.0.1 --port 9000
```

| Situation | Action / constraint |
| --- | --- |
| Port occupied | Select another `--port` from `1` to `65535` |
| Invalid address | Use an IPv4/IPv6 address, not a hostname, multicast address, or IPv4 broadcast address |
| Phone cannot connect | Check the private network, PC address, port, VPN interface, and firewall scope |
| SimConnect selection fails | `--source simconnect` is not implemented. Demo never replaces it automatically |

## Project guide

| I want to… | Read |
| --- | --- |
| Run checks or test a phone | [Testing](docs/testing/index.md) |
| Contribute a change | [Contributing](CONTRIBUTING.md) |
| Use the Rust library or understand the design | [Architecture](docs/architecture.md#application-api) |
| Consume telemetry | [Telemetry contract](docs/telemetry-contract.md) |
| Review scope and future work | [Project brief](docs/project-brief.md) · [Roadmap](docs/roadmap.md) |
| Understand the stack choice | [ADR 0001](docs/adr/0001-rust-http-sse.md) |
| Track work | [GitHub Issues](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues) |

[MIT license](LICENSE).
