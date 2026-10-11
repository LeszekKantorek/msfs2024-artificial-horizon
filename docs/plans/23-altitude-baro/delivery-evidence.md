# Altitude and pressure delivery evidence

## Tested source

| Field | Value |
| --- | --- |
| Issue | [#23](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/23) |
| Implementation commit | `8340204` |
| Date | 2026-10-11 |
| Platform | Windows x64, MSVC |
| Rust | Stable 1.99.0 (`b940084d7`, LLVM 23.1.1) |
| Node | 24.21.0 |
| Browser | Desktop Edge 155.0.4283.45 |

The final Rust and Node checks ran at the implementation commit.
Both browser scripts checked the same source and fixtures before that commit.
Only documentation, context, and this evidence record changed after their passing runs.
Release validation used `target/x86_64-pc-windows-msvc/release/msfs2024-artificial-horizon.exe`.

## Validation

| Check | Result | Evidence |
| --- | --- | --- |
| Format check | Passed | `cargo fmt --all -- --check` |
| Clippy | Passed | All targets, locked dependencies, warnings denied |
| Rust tests | 45 passed | All targets, including typed altitude, independent fields, deterministic fixtures, two-client SSE, and lifecycle cleanup |
| Node tests | 47 passed | Optional validation, signed geometry, drum carries/boundaries, reconnect/resume, and scheduler expiry |
| Release build | Passed | Locked optimized MSVC build |
| Fixture browser script | Passed | Production/full arrangement, local loss/overflow, stale/resize recovery, stable nodes, painted bounds, and 24 altitude screenshots |
| Release browser script | Passed | Embedded modules/MIME, Rust SSE values, SVG drums/sign, pressure formatting, and restart recovery |
| Independent code review | No outstanding findings | Screenshot timing corrected and distinct backgrounds with readable long values inspected in follow-up |

Representative captures are in ignored `.local/pfd-browser/`:

* `altitude-production-326x246--9980-40.png`
* `altitude-production-326x246-99980--40.png`
* `altitude-layout-320x480--9980--40.png`
* `altitude-layout-568x240-99980-40.png`

## Limits and pending acceptance

* The sandbox blocked local TCP access in the first Rust run with error 10013.
* The same suite passed in the permitted host environment without product changes.
* Edge measured a 2-pixel overlap in the initial pressure value/unit layout.
* A 32-pixel pressure background and revised text positions passed the painted-bounds checks.
* The screenshot test now waits for the requested pitch direction and altitude before capture.
* Desktop results establish demo behavior and do not establish simulator integration or real-phone acceptance.

Remaining device checks follow [altitude acceptance](../../testing.md#altitude-and-pressure-acceptance):

* [ ] Real iOS Safari: presentation, bars/safe areas, rotation, local unavailable states, overflow, reconnect, and lock/resume.
* [ ] Real Android Chrome: the same presentation and lifecycle checks.

Record device, OS, browser/version, tested commit, observations, and pass/fail in the issue or PR.
Keep the implementation PR draft until required device evidence passes.
