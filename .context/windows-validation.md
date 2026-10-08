---
description: Handle Windows Smart App Control failures during Rust checks without bypassing the owner's stop rule.
status: active
---

## Required response

1. If a command reports Smart App Control error 4551, immediately tell the user which command and file were blocked.
2. Stop further execution until the user explicitly instructs resumption.
3. Do not retry the command, escalate it to bypass SAC, or change Windows security settings.

## Diagnostic evidence and limits

- During the 2026-10-07 session, CodeIntegrity event 3077 confirmed that SAC blocked Rust tools, build scripts, test executables and a generated tokio-macros DLL.
- Rust's E0463 missing-crate error was also observed when Windows blocked loading that existing DLL, so do not assume a dependency upgrade fixes it.
- Current SAC state must be checked rather than inferred from earlier sessions.
- The stop rule was explicitly requested by the owner in the issue #3 planning session.

## Sources

- [Testing procedures](../docs/testing.md) define project checks and hardware evidence limits.
- [Microsoft SAC FAQ](https://support.microsoft.com/en-us/windows/security/threat-malware-protection/smart-app-control-frequently-asked-questions) describes trust decisions and settings.
