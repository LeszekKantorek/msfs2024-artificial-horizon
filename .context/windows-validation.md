---
description: Respond to Windows Smart App Control failures during Rust checks.
status: active
---

## Required response

1. On Smart App Control error 4551, immediately report the command and blocked file.
2. Stop further execution until the user explicitly authorizes resumption.
3. Do not retry, escalate to bypass SAC, or change Windows security settings.

## Evidence and limits

| Observation, 2026-10-07 | Consequence |
| --- | --- |
| CodeIntegrity event 3077 recorded blocked Rust tools, build scripts, test executables, and a generated `tokio-macros` DLL | Check current SAC state instead of assuming it |
| Rust E0463 occurred when Windows blocked that existing DLL | Do not assume a dependency upgrade fixes missing-crate errors |

Sources: explicit stop requirement from issue #3 planning, 2026-10-07, and [Microsoft SAC FAQ](https://support.microsoft.com/en-us/windows/security/threat-malware-protection/smart-app-control-frequently-asked-questions).
See [testing](../docs/testing.md) for checks and hardware evidence limits.
