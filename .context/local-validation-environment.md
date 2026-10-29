---
description: Diagnose missing browser tooling, sandbox restrictions, executable replacement, or a browser that cannot reach a local fixture.
status: active
---

* Read [Windows validation](windows-validation.md) first if the failure reports SAC error 4551 or a blocked Rust tool or DLL.
* Preserve its stop requirement. The recoveries below do not authorize bypassing SAC.
* Use [testing](../docs/testing/index.md) for check commands and browser fixture setup.

## Executable replacement fails

1. Record the failed command, error code, and executable path.
2. Check whether a running server from this task holds that exact executable.
3. Stop that task-owned server normally before rebuilding, or use a separate Cargo target directory for validation.
4. Run the affected check again and record its result.

> Error 5 alone does not prove a file lock or SAC failure. Do not stop unrelated processes.

## Playwright module is missing

If `PLAYWRIGHT_MODULE` cannot be resolved, check for an existing Playwright installation before installing another copy. In Codex desktop, the workspace dependency runtime can provide a Node packages directory; set `PLAYWRIGHT_MODULE` to its `playwright` package. Otherwise follow the [browser setup](../docs/testing/pfd.md#responsive-efi-layout-and-desktop-checks). Availability and paths depend on the current environment; do not assume an earlier cache path still exists.

## Browser cannot reach a fixture

1. Check the fixture's startup output, listening address, and port.
2. Check reachability from the browser's execution environment.
3. If execution environments differ, run the browser and loopback fixture in the same permitted environment.
4. Verify both page loading and the intended browser scenario.

> A browser launch failure and a fixture connection timeout are different symptoms. Do not change security settings to resolve them.

An `EACCES` connection failure to `127.0.0.1` can indicate that the current sandbox blocks loopback, even after Playwright loads successfully. Check the error and fixture environment first; use an approved environment with loopback access when needed. Do not treat a missing module, a browser launch error, and a blocked connection as the same failure.

## Rust tests encounter sandbox restrictions

Local socket or filesystem permission failures can prevent integration tests from exercising their intended behavior. Record the failing tests and native errors, then verify whether the execution environment permits their required loopback and file operations. Use an appropriately approved environment when those operations are blocked; rerun the affected tests and retain their actual outcomes. Do not infer that every permission error is caused by sandboxing.

## Evidence

* During #20 validation, error 5 prevented removal of the running debug executable. Checks passed with separate target output.
* Browser launch and fixture reachability failures occurred across sandbox boundaries. Running the fixture and browser together enabled the checks.
* During #87–#89, the configured Playwright module was absent, while a bundled workspace copy worked. A separate sandboxed run then failed with loopback `EACCES`; the browser suite and Rust socket/filesystem tests passed in an environment with the necessary access.
* These observations establish conditional recovery paths, not a universal cause for either symptom.
