# Real-device acceptance

Shared phone scenarios, controlled fixture setup, and per-device evidence.

## Manual acceptance matrix

Run on real iOS Safari and Android Chrome.
Desktop emulation does not establish mobile lifecycle compatibility.

| Scenario | Expected result |
| --- | --- |
| Portrait / landscape / 320 CSS px | Readable scales/status without horizontal scrolling |
| Simulator absent / flight not loaded | Disconnected or waiting, no fabricated live attitude |
| Pause / menu / flight exit / simulator restart | Explicit state, then automatic recovery on fresh data |
| Network loss / Wi-Fi recovery | Reconnecting/stale, then current state without reload |
| HTTP server restart | Reconnect accepts sequence restart |
| Background / phone lock / resume | Fresh sample required before live |
| Two phones / deliberately slow receiver | Independent delivery, bounded memory |
| Source stops, SSE remains open | Stale within one second on active page. Heartbeats cannot mask it |
| Invalid values | Visible invalid state, no level-flight fallback |

Apply the [attitude comparison](../efi/attitude.md#simulator-comparison-scope) and other affected feature scenarios in addition to this shared matrix.

## Real-phone acceptance procedure

Use this procedure with the affected feature's scenarios and evidence owner.
Run it separately on actual iOS Safari and Android Chrome.
Desktop emulation does not establish device acceptance.

1. Identify the feature issue, tested build, and applicable scenarios before the run.
2. Start the compiled demo on the private LAN using the [phone setup](../../README.md#use-your-phone). Open its printed URL on each phone.
3. Exercise the feature's cycle and controlled inputs, inspecting values, symbols, and local failure marks against both sky and ground.
4. Check portrait, short landscape, rotation, browser-bar changes, safe areas, and undistorted, readable indications without scrolling or collisions.
5. Use the [minimum usable dimensions](../project-brief.md#mobile-presentation) and the feature's painted-bounds criteria when checking layout.
6. Exercise lock/unlock, app switching, and browser hide/restore. Require a fresh sample before valid indications return.
7. Disable and restore Wi-Fi. Check the unavailable cover and retry status, followed by fresh-data recovery without reload.
8. Stop and restart the server on the same port. Require recovery despite sequence restart and no duplicate subscriptions.
9. Record each device separately using the template below. Put the feature's observations and screenshots in its issue or PR.
10. Leave missing or failing device criteria unchecked. Keep the owning issue open until both required device results support acceptance.

Check uninterrupted freshness as well as recovery: no frozen valid-looking indication may survive loss or resume.
For simulator-specific checks, also apply the [manual acceptance matrix](#manual-acceptance-matrix).

Evidence template for the issue or PR:

| Field | Value |
| --- | --- |
| Build commit / local changes | |
| Date / viewport / zoom | |
| Phone model | |
| OS version | |
| Safari / Chrome version | |
| Network and server address | |
| Feature / controlled inputs / screenshots | Pass/fail, observations |
| Demo poses and directions | Pass/fail, observations |
| Portrait / landscape / safe areas | Pass/fail, observations |
| Lock / background / resume | Pass/fail, observations |
| Wi-Fi loss / recovery | Pass/fail, observations |
| Server restart / recovery | Pass/fail, observations |

### Phone fixture setup

Use controlled inputs for repeatable bounds, partial failures, or visual scenarios.
Start the development fixture on the PC's specific private LAN IP:

```powershell
$env:PFD_FIXTURE_HOST = '<PC LAN IP>'
node tests/browser/helpers/browser-fixture-server.mjs
```

Open the printed address on the phone. Use `/` for production composition and `/layout.html` for all 17 illustration elements.
Enter the feature's JSON lines in the fixture server terminal.
The host defaults to loopback and accepts IP addresses only.
HTTP has no fixture-control route, and fixture assets are absent from the Rust binary.
Restrict firewall access to the trusted private subnet. Remove `PFD_FIXTURE_HOST` from the environment after the phone run.
