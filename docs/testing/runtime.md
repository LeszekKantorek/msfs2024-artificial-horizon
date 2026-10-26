# HTTP, telemetry, and browser lifecycle checks

Startup, shutdown, subscriptions, controlled time, and deterministic demo fixtures.

## HTTP startup checks

Prerequisites: Rust stable, rustfmt, and Clippy on Windows x64 MSVC.
Use committed `Cargo.lock`. These checks require no simulator or SDK.

1. Run the [automated checks](automated.md#automated-checks), including the build and tests.
2. Record the compiler version with local results.

   ```powershell
   rustc --version --verbose
   ```

3. Start the server.

   ```powershell
   cargo run --locked --bin msfs2024-artificial-horizon
   ```

4. Check the endpoints and CLI behavior below.
5. Press `Ctrl+C`.
6. Check successful exit.
7. Restart on the same port.

| Check | Expected result |
| --- | --- |
| `http://127.0.0.1:8080/` | HTML |
| `http://127.0.0.1:8080/styles.css` | CSS |
| `http://127.0.0.1:8080/health` | JSON `{"status":"ok"}` |
| `--help`, `--version` | Successful output |
| Invalid arguments | Actionable errors |
| Page without telemetry | Unavailable indication, no credible attitude |
| Explicit LAN bind: private interface or `0.0.0.0` | Printed URL uses the correct interface |

Automated lifecycle tests cover normal shutdown and cancellation:

* Library tests signal shutdown independently and rebind within a five-second completion timeout.
* Cancellation tests abort a task or drop a polled server future after publication with two active SSE clients.
* Check producer/subscription closure, client EOF, and port reuse after cancellation.
* Permit the final pending sample before subscription closure.
* Public configuration rejects port zero.
* Internal listener tests use OS-assigned ports to avoid collisions.

> Desktop checks do not establish real-phone or simulator compatibility.

## SSE and browser controller checks

```powershell
cargo test --locked --test http_sse
node --test tests/*.test.mjs
```

Node checks require Node.js 24, without npm install or a browser framework.
Run these checks locally on Windows x64 MSVC.

| Suite | Coverage / method |
| --- | --- |
| HTTP SSE | Framing/headers, immediate state, ignored `Last-Event-ID`, invalid/unavailable/stale states, delivery-time age, independent slow consumers, heartbeat comments, channel closure |
| HTTP timing | Real Axum bodies and a paused monotonic clock. Do not collect an infinite stream |
| Socket unit tests | Subscriber release and write/shutdown deadlines with bounded duplex transport |
| Local TCP | Two active SSE connections and port reuse after shutdown |
| DOM-free imports | `tests/helpers.test.mjs` imports visual owners/helpers/telemetry without `document` and exercises meaningful pure behavior |
| Frame scheduler | Latest-frame/layout coalescing, coherent resize and render, expiry before draw, cancellation on data loss, resize without renewing age, fresh identical values |
| Panel/status | Complete read-only frame delegation, shared attitude clip through resize/expiry, status without a sample, stale obscuring, separate source/transport state and announcements |

Apply the [client and validation suites](../efi/telemetry.md#client-and-validation-suites) alongside these integration checks.

### Desktop exploration

1. Start demo and open the page.
2. Check full-panel sky/ground, no source badge, and no top status bar.
3. Stop the server.
4. Check connection loss and retry indication.
5. Restart on the same port without reloading the page.
6. Check automatic recovery despite sequence restart.
7. Hide and restore the page.
8. Check that live attitude requires a new snapshot.
9. Inspect the network stream for named telemetry events and no duplicate subscriptions.

> Label these results as desktop evidence, not iOS/Android compatibility.

## Telemetry and demo checks

```powershell
cargo test --locked --test telemetry
```

These tests require no simulator or SDK.

### Demo behavior

Feature pages own the [attitude/turn cycle](../efi/attitude.md#demo-behavior) and [IAS/GS cycle](../efi/airspeed.md#demo-behavior).
The following rules apply to the shared producer:

| Property | Expected behavior |
| --- | --- |
| Sampling | One fresh sample every 50 ms, including unchanged poses. Skip missed ticks |
| Initial state | `waiting`, then `live` on fresh data |
| Expiry | No accepted sample for 1,000 ms makes attitude stale |
| Source failure | Never substitute demo for unavailable SimConnect |

### Fixtures and tests

| Fixture / suite | Coverage |
| --- | --- |
| `tests/fixtures/snapshots.json` | v1 wire shape, including null attitude in unavailable states |
| Telemetry tests | Age, stale threshold, fresh identical samples, cadence, skipped ticks, normalized bounds, roll wrap, non-finite rejection, source identity, slow independent consumers |
| Library tests | Sequence exhaustion, shutdown ownership, subscription closure, listener reuse |

Maintain fixtures and timing checks as follows:

* Keep fixtures synchronized with contract changes.
* Reuse fixtures for future browser tests.
* Use a paused monotonic Tokio clock instead of wall-clock sleeps for timing checks.
