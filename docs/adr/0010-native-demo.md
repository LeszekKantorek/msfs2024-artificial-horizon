# ADR 0010: Native Demo playback

Decision status: Accepted.

## Context

Demo must exercise native loading, definitions, dispatch, and lifecycle acquisition without requiring a simulator. Both Demo and the official SimConnect DLL use the same receiver.

## Decision

The workspace contains a Windows x64 Rust cdylib package producing `demo.dll`. Scenario instructions use JSONL and drive native observations through the existing SimConnect receiver, decoder, lifecycle gates, Publisher, HTTP/SSE, and EFI. The DLL owns its clock, request tables, notifications, receive buffers, and injected faults. The Demo DLL selects and validates the scenario from the host process arguments on its first Open. This couples Demo playback to a host executable that passes `--scenario`; the official DLL remains unchanged. The [Demo guide](../sources/demo.md#native-demo-playback) and [native consumer contract](../sources/simconnect.md#common-native-consumer) define behavior and evidence limits.

The official DLL uses manual bindings without SDK build dependencies. [ADR 0006](0006-simconnect-manual-ffi.md) defines native integration and [ADR 0007](0007-application-composition.md) defines application composition.

Native-input playback exercises the receiver. Recorded-session conversion and schema 4/5 import are excluded; forward input-to-output evidence is required. The baseline scenario does not supply fictional native values for VFE/VNO/VNE or VX/VY/VG. Presentation requirements and full-profile fixtures remain valid. [ADR 0008](0008-partial-airspeed-profile.md) governs independently optional airspeed data.

## Consequences and limits

* Native registration is independent of scenario coverage. Missing values remain unavailable, not zero.
* An increasing DLL clock loops scenario steps without resetting lifecycle state. Request semantics govern observations.
* The Demo is an independently implemented test source, not proof of simulator compatibility. Empty-queue, pause delivery, overflow, indexed spellings, and native layouts require the bounded checks in the source guides.
* A separate DLL must be built and delivered with the application. The shared receiver and DLL-path CLI implement this decision.
