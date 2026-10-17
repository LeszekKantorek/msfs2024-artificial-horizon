# Automated application checks

Windows/MSVC commands and cross-component coverage boundaries.

## Automated checks

Run these commands from the repository root on Windows x64 MSVC.
They match the checks in [CI](../../.github/workflows/ci.yml).

Complete the [environment setup](../../CONTRIBUTING.md#environment-setup) first.
Use committed `Cargo.lock` with `--locked`.
The checks require no simulator or SDK.

Use one target for Clippy, the build, and tests:

```powershell
$env:RUSTUP_TOOLCHAIN = 'stable'
$env:CARGO_BUILD_TARGET = 'x86_64-pc-windows-msvc'
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
node --test tests/*.test.mjs
cargo build --locked --release
```

CI runs these checks in the `Build & Test` job without restoring artifacts from previous workflow runs.
Cargo commands reuse artifacts in `target` within the job and during local development.
The final step builds the release executable at `target/x86_64-pc-windows-msvc/release/msfs2024-artificial-horizon.exe`.
Tests use the debug profile.
The release build compiles optimized artifacts separately.

> The Windows SimConnect module contains no SDK integration. Passing these checks does not establish simulator compatibility.
> Future SDK-dependent checks require documented prerequisites and distribution constraints.

The [PR acceptance control](../pr-acceptance.md#validate-changes-to-the-control) has a separate Node suite and runs on Linux.
Its tooling dependencies do not affect browser controller checks or the application build.

### Coverage boundaries

| Boundary | Required coverage |
| --- | --- |
| Library/CLI | Typed configuration without process parsing, help/version, actionable invalid-value errors |
| Model | Units/signs, roll wrap, invalid numbers, serialization, monotonic age, fixtures independent of implementation formulas |
| Providers | Deterministic trajectories, unchanged fresh samples, lifecycle transitions, reconnect backoff, shutdown/handle ownership |
| HTTP/SSE | Health independent of source, immediate full snapshot, framing, MIME/cache headers, reconnect without replay, shared acquisition, two clients, slow consumers, cleanup |
| Browser | Pure transforms, message validation, separate source/transport status, receive timeout, fresh data after background/resume |

See each [feature's tests](../efi/index.md#features) for geometry expectations.
