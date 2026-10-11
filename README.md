# MSFS2024 Artificial Horizon

An aircraft attitude display for your phone. Start the Rust server on Windows,
then open its address in a browser on the same network.

**Windows x64** · **Read-only browser display** · **No phone app or account**

> **Try the demo today.** The current executable displays synthetic attitude, IAS, GS, altitude, selected altitude with alerts, pressure setting, slip/skid, and turn rate.
> Vertical speed, heading/track, and MSFS2024 integration remain planned. This is not a real-flight instrument.

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
Expect a moving attitude display, a left-side airspeed tape, ground speed, and a right-side altitude tape with pressure setting.
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

## What to expect

| Behavior | Current demo |
| --- | --- |
| Instrument | Responsive attitude, 2.5-degree pitch ladder, extreme-pitch chevrons, bank scale, fixed aircraft reference, slip/skid and turn rate |
| Motion | Repeating 18-second cycle: seven original attitudes, then +70/-50-degree pitch. Centered and both-direction slip/skid; +/-3 deg/s turns |
| Speed | IAS accelerates from 0 to 150 kt, holds, then decelerates. GS follows a different profile up to 170 kt |
| Speed display | Rolling IAS digits, tape labels every 10 kt, colored speed ranges, R/X/Y/G references, and a magenta six-second trend. Separate rounded GS in kt |
| Speed warnings | Red current-speed pointer at or above 140 kt. Yellow IAS digits when the rising trend predicts reaching 140 kt |
| Speed unavailable | `X` means unavailable, `OVR` means above 999 kt. Configured references appear above GS when IAS is unavailable |
| Altitude | Fixed 8 CSS px spacing per 20 ft. Labels every 100 ft. Taller panels show more altitude. Signed readout: -9999 to 99999 ft. |
| Pressure | Read-only scenario setting below altitude, with two decimal places in inHg |
| Selected altitude | Read-only scenario target above altitude, with a tape bug pinned at either edge. `X` means unavailable. `OVR` means outside -9999 to 99999 ft |
| Altitude alerts | Five-second flashing on entry within 1000 and 200 ft. Entry within +/-200 ft arms deviation. Leaving it flashes yellow for five seconds, then stays yellow until return or reset |
| Altitude/pressure unavailable | Independent `X` marks; `OVR` beyond each display limit |
| Data | Synthetic samples at 20 Hz, without a source badge |
| Presentation | Sky/ground across the full panel, centrally clipped pitch scale, no top status bar |
| Connection loss | Opaque full-panel cover with the reason and automatic retry after two seconds |
| Stale or unavailable data | Explicit status, no fabricated level-flight reading |
| Background/resume | A fresh snapshot is required before live attitude returns |
| Sizing | No fixed aspect ratio; minimum 320 CSS px width and 240 CSS px usable height after browser bars and safe areas |
| Old snapshots | Attitude-only snapshots display attitude with unavailable IAS, GS, altitude, selected altitude, pressure, slip/skid and turn rate |

> Real iOS Safari and Android Chrome acceptance requires [device evidence](docs/testing.md#attitude-display-real-device-phone-acceptance).
> Issue #4 owns the original attitude acceptance, [#20](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/20) owns the attitude/turn extension, and [#37](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/37) owns layers and background opacity.

[#21](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/21) owns IAS/GS acceptance.
[#22](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/22) owns range, reference, and trend acceptance.
The speed profile is fictional: VS0/VS1 40/50, VFE/VNO/VNE 85/110/140, and VR/VX/VY/VG 55/65/75/90 kt.
These values are not verified performance of a real aircraft. See the [speed contract](docs/telemetry-contract.md#optional-airspeed-ranges-and-trend).
[#23](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/23) owns altitude and pressure acceptance.
The altitude demo repeats every 90 seconds. It starts with 18 seconds of signed digit sweeps.
Two 36-second approaches and departures follow, at selected targets of 10000 and 1000 ft.
The other indications keep their existing 18-second cycles. Transitions are not a physical flight model.
[#24](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/24) owns selected-altitude and alert acceptance on real phones.
First samples after target changes or recovery establish the alert state without flashing.
The planned G5-inspired PFD adds vertical speed and heading/track indications.
See the [project brief](docs/project-brief.md) for coverage and exclusions, and the [roadmap](docs/roadmap.md) for delivery order.

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
| Run checks or test a phone | [Testing](docs/testing.md) |
| Contribute a change | [Contributing](CONTRIBUTING.md) |
| Use the Rust library or understand the design | [Architecture](docs/architecture.md#application-api) |
| Consume telemetry | [Telemetry contract](docs/telemetry-contract.md) |
| Review scope and future work | [Project brief](docs/project-brief.md) · [Roadmap](docs/roadmap.md) |
| Understand the stack choice | [ADR 0001](docs/adr/0001-rust-http-sse.md) |
| Track work | [GitHub Issues](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues) |

[MIT license](LICENSE).
