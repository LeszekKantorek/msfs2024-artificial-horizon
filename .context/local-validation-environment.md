---
description: Diagnose error 5 when replacing a running executable, or a browser that cannot reach a local fixture.
status: active
---

* Read [Windows validation](windows-validation.md) first if the failure reports SAC error 4551 or a blocked Rust tool or DLL.
* Preserve its stop requirement. The recoveries below do not authorize bypassing SAC.
* Use [testing](../docs/testing.md) for check commands and browser fixture setup.

## Executable replacement fails

1. Record the failed command, error code, and executable path.
2. Check whether a running server from this task holds that exact executable.
3. Stop that task-owned server normally before rebuilding, or use a separate Cargo target directory for validation.
4. Run the affected check again and record its result.

> Error 5 alone does not prove a file lock or SAC failure. Do not stop unrelated processes.

## Browser cannot reach a fixture

1. Check the fixture's startup output, listening address, and port.
2. Check reachability from the browser's execution environment.
3. If execution environments differ, run the browser and loopback fixture in the same permitted environment.
4. Verify both page loading and the intended browser scenario.

> A browser launch failure and a fixture connection timeout are different symptoms. Do not change security settings to resolve them.

## Evidence

* During #20 validation, error 5 prevented removal of the running `target/debug/main.exe`. Checks passed with separate target output.
* Browser launch and fixture reachability failures occurred across sandbox boundaries. Running the fixture and browser together enabled the checks.
* These observations establish conditional recovery paths, not a universal cause for either symptom.

Source: session `01a121eb-3bf2-70c1-add4-5cd56f3fe761`, 2026-10-09, reviewed in [#41](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/41).
