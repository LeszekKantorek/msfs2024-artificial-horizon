# ADR 0009: Native Demo playback

Decision status: Accepted.

## Context

The current Demo provider publishes normalized samples directly. It does not exercise native loading, definitions, dispatch, or lifecycle acquisition. The accepted direction exercises the same receiver for Demo and the official SimConnect DLL while retaining simulator-free startup.

## Decision

The workspace contains a Windows x64 Rust cdylib package producing `demo.dll`. Scenario instructions use JSONL and drive native observations through the existing SimConnect receiver, decoder, lifecycle gates, Publisher, HTTP/SSE, and EFI. The DLL owns its clock, request tables, notifications, receive buffers, and injected faults. The Demo DLL selects and validates the scenario from the host process arguments on its first Open. This couples Demo playback to a host executable that passes `--scenario`; the official DLL remains unchanged. The [Demo guide](../sources/demo.md#native-demo-playback) and [native consumer contract](../sources/simconnect.md#planned-common-native-consumer) define behavior and evidence limits.

This amends [ADR 0006's bindings and runtime](0006-simconnect-manual-ffi.md#bindings-and-runtime): Demo uses a project-owned DLL and a separate workspace package, while the official DLL still uses the existing manual bindings without SDK build dependencies. It amends [ADR 0007's component boundaries](0007-application-composition.md#component-boundaries): both source identities use the same native receiver. HTTP-first startup, one active publisher, source-failure isolation, and coordinated shutdown remain valid. No fallback to Demo is introduced.

The earlier planned YAML and full normalized-snapshot playback are replaced by native-input JSONL instructions. They would have retained the direct provider and its full fictional profile, leaving the native receiver unexercised. Recorded-session conversion and schema 4/5 import are removed; forward input-to-output evidence is required instead. The default cannot supply fictional native values for VFE/VNO/VNE or VX/VY/VG. Existing presentation requirements and full-profile fixtures remain valid. [ADR 0008](0008-partial-airspeed-profile.md) continues to govern independently optional airspeed data; its full Demo profile consequence describes the earlier provider.

## Consequences and limits

* Native registration is independent of scenario coverage. Missing values remain unavailable, not zero.
* An increasing DLL clock loops scenario steps without resetting lifecycle state. Request semantics govern observations.
* The Demo is an independently implemented test source, not proof of simulator compatibility. Empty-queue, pause delivery, overflow, indexed spellings, and native layouts require the bounded checks in the source guides.
* A separate DLL must be built and delivered with the application. The shared receiver and DLL-path CLI implement this decision.
