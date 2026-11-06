# ADR 0009: Agent-operated live development through the production application

Decision status: Accepted, amended 2026-11-05.

## Context

An agent needs to observe the production SimConnect path while MSFS2024 and a flight are already running. The application owns the SimConnect connection and exposes telemetry through HTTP/SSE. The workflow does not launch or control the simulator and does not attach to an existing application process.

## Decision

Use a Node.js recorder around a newly launched application. The agent builds the application, selects its arguments and local endpoint, and inspects the resulting data. The recorder starts the application, records the HTTP response body unchanged to one supplied file, and stops after the requested time or message limit. It requests application shutdown by writing `stop` to standard input.

The recorder does not validate the payload, run a browser, read diagnostics, or correlate observations. The agent may use the existing optional diagnostic log and browser procedures separately. The [SimConnect observation procedure](../sources/simconnect.md#bounded-development-observation) describes the current command and its limits.

Preserve one production provider, explicit Demo selection, existing telemetry contracts, and coordinated application shutdown. Generated captures, SDK binaries, and machine-specific paths stay outside Git.

## Rationale and evidence

Reusing the production application observes its actual acquisition and HTTP delivery path without a separate SimConnect client. Raw capture leaves interpretation and additional checks with the agent. Recorded SSE alone does not establish simulator accuracy, browser receipt, or display behavior. Demo and fixture runs do not establish live simulator acceptance.

Independent native acquisition remains deferred until a concrete diagnostic problem requires it. The native integration and application lifecycle remain governed by [ADR 0006](0006-simconnect-manual-ffi.md) and [ADR 0007](0007-application-composition.md).

## Previous decision

The original decision proposed a broader harness with build and executable identification, readiness checks, diagnostic correlation, browser observations, sampling, and reports. The 2026-11-05 amendment replaced that harness with raw capture. Earlier schema 5 diagnostic records and previous test results remain historical evidence; the current diagnostic journal uses schema 4.
