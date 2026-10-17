---
description: Select contracts and validation when changing behavior across providers, telemetry, HTTP, and browser components.
status: active
---

## Select a change across components

Select a row by the behavior being changed, then follow its contracts and checks.
Apply the [validation scope matrix](../docs/testing.md#validation-by-change-scope) to every affected boundary.
Use the [decision map](architecture-decisions.md#find-the-governing-decision) when reconsidering an accepted choice.

| Change | Read first | Validation | Extend only when relevant |
| --- | --- | --- | --- |
| Change sample expiry or unavailable indications | [Server and browser ownership](../docs/architecture.md#server-and-browser-ownership), [freshness and age](../docs/telemetry-contract.md#freshness-and-status) | [Telemetry checks](../docs/testing.md#telemetry-and-demo-checks), [SSE and controller checks](../docs/testing.md#sse-and-browser-controller-checks) | [Frame lifecycle](../docs/pfd-presentation.md#panel-interface-and-frame-lifecycle) when expiry affects scheduled drawing |
| Change reconnect or background/resume behavior | [Session lifecycle](../docs/architecture.md#session-and-lifecycle), [browser lifecycle decision](../docs/adr/0002-sse-connection-lifecycle.md#browser-lifecycle) | [SSE and controller checks](../docs/testing.md#sse-and-browser-controller-checks), [real-phone procedure](../docs/testing.md#real-phone-acceptance-procedure) | [Unavailable presentation](../docs/pfd-presentation.md#unavailable-status) when changing the cover or status display |
| Change shutdown, cancellation, or producer failure | [Server ownership](../docs/architecture.md#server-and-browser-ownership), [application API](../docs/architecture.md#application-api) | [HTTP startup and shutdown checks](../docs/testing.md#http-startup-checks) | [Socket deadlines](../docs/adr/0002-sse-connection-lifecycle.md#socket-deadlines) when open connections can delay shutdown |
| Change delivery to slow clients or SSE framing | [State distribution](../docs/architecture.md#state-distribution), [wire framing](../docs/telemetry-contract.md#framing) | [SSE checks](../docs/testing.md#sse-and-browser-controller-checks) | [Socket deadlines](../docs/adr/0002-sse-connection-lifecycle.md#socket-deadlines) for stalled writes; [device measurements](../docs/testing.md#measurements-and-evidence) for latency claims |
| Add or change a telemetry field | [Model boundary](../docs/telemetry-contract.md#model-boundary), [field conventions](../docs/telemetry-contract.md#fields-and-conventions), [compatibility](../docs/telemetry-contract.md#compatibility) | [Telemetry fixtures](../docs/testing.md#fixtures-and-tests) | [Presentation map](../docs/pfd-presentation.md#find-the-affected-boundary) and [feature checks](../docs/testing.md#find-the-required-procedure) when the field affects an indication |
| Add or replace a telemetry provider | [Component boundaries](../docs/architecture.md#boundaries), [library telemetry interface](../docs/architecture.md#library-telemetry-interface) | [Telemetry checks](../docs/testing.md#telemetry-and-demo-checks), [startup checks](../docs/testing.md#http-startup-checks) | [Integration uncertainty](../docs/architecture.md#integration-uncertainty) and [simulator acceptance](../docs/testing.md#manual-acceptance-matrix) for a real simulator source |
| Move rendering, scheduling, or visual-object responsibilities | [Panel boundary](../docs/architecture.md#panel-boundary), [presentation map](../docs/pfd-presentation.md#find-the-affected-boundary) | [Desktop layout checks](../docs/testing.md#responsive-pfd-layout-and-desktop-checks), [composition acceptance](../docs/testing.md#modular-and-layered-pfd-acceptance) | [Telemetry operations](../docs/pfd-presentation.md#telemetry-operations) when changing acceptance or invalidation of samples |
| Change the listen address or network exposure | [Deployment](../docs/architecture.md#deployment), [application API](../docs/architecture.md#application-api) | [HTTP startup checks](../docs/testing.md#http-startup-checks) | [Security boundaries](security-review.md#project-boundaries) when changing access or trust; [phone setup](../README.md#use-your-phone) when changing LAN instructions |

Sources: the linked architecture, contracts, decisions, and validation procedures. This map routes changes without adding requirements.
