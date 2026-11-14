# Demo scenario catalog

The suite contains a narrated flight, focused availability and lifecycle inputs, and selected fault scenarios. The [Demo contract](demo.md#native-demo-playback) defines playback; scenario files own values, durations, event order, and fault parameters. Each scenario can supply a subset of signals or only events. The [native error contract](simconnect.md#dispatch-and-error-limits) owns error outcomes.

| Scenario | Purpose and assumptions | Expected completion | File | Visual regression |
| --- | --- | --- | --- | --- |
| Baseline boundaries | Preserve applicable current native-input cycles and independent boundary checks | Repeats until shutdown | [Baseline scenario](../../demo-scenarios/demo-scenario.jsonl) | Not specified |
| Flight tour | Fictional flight progression, both pause modes, and recoverable connection changes; no flight dynamics or ground-contact model | Repeats until shutdown | [Flight tour](../../demo-scenarios/flight-tour.jsonl) | No. Only unit tests and manual acceptance testing |
| Simulator lifecycle | Connection availability, inactive flight, pause, flight change, and reconnect | Recovers where scheduled and repeats | [Simulator lifecycle](../../demo-scenarios/simulator-lifecycle.jsonl) | Not specified |
| Availability gaps | Equal fresh samples, signal gaps, target gating, and recovery | Grouped optional values can become unavailable independently | [Availability gaps](../../demo-scenarios/availability-gaps.jsonl) | Not specified |
| IAS only | Sparse native IAS without attitude | Stays unavailable for live display | [IAS only](../../demo-scenarios/ias-only.jsonl) | Not specified |
| Event only | Lifecycle events without instrument tracks | Repeats until shutdown | [Event only](../../demo-scenarios/event-only.jsonl) | Not specified |
| No signals | Initial lifecycle without instrument tracks | Repeats until shutdown | [No signals](../../demo-scenarios/no-signal.jsonl) | Not specified |
| Startup retry | First Open fails; the following attempt recovers | Continues with live data and HTTP available | [Scenario](../../demo-scenarios/errors/startup-retry.jsonl) | Not specified |
| Definition failure | Fail a mapping registration | Producer terminates and HTTP remains available | [Scenario](../../demo-scenarios/errors/optional-definition.jsonl) | Not specified |
| Legacy optional request failure | Selector targets a former single-field request | Not applicable to the grouped request | [Scenario](../../demo-scenarios/errors/optional-request.jsonl) | Not specified |
| Legacy optional exception | Refers to a former optional request sequence | Requires a grouped-request fixture before use | [Scenario](../../demo-scenarios/errors/optional-exception.jsonl) | Not specified |
| Exception before sample | Deliver a correlated failure before a same-frame sample and inject a later stale packet | Producer terminates and late data cannot restore it | [Scenario](../../demo-scenarios/errors/exception-before-sample.jsonl) | Not specified |
| Definition exception | Correlate an asynchronous failure with a definition entry | Producer terminates and HTTP remains available | [Scenario](../../demo-scenarios/errors/definition-exception.jsonl) | Not specified |
| Uncorrelated exception | Deliver a native exception without a matching request | Producer ends while HTTP remains available | [Scenario](../../demo-scenarios/errors/uncorrelated-exception.jsonl) | Not specified |
| Lost packet ID | With diagnostic logging enabled, fail correlation before a later exception | Producer ends while HTTP remains available | [Scenario](../../demo-scenarios/errors/packet-id-exception.jsonl) | Not specified |
| Missing sent call | Reference an unavailable call from an exception record | Scenario error ends the producer while HTTP remains available | [Scenario](../../demo-scenarios/errors/missing-sent-call.jsonl) | Not specified |
| Failed sent call | With diagnostic logging enabled, reference a failed call from an exception record | Scenario error ends the producer while HTTP remains available | [Scenario](../../demo-scenarios/errors/failed-sent-call.jsonl) | Not specified |
| Packet ID failure | Fail one packet ID lookup | Producer terminates and HTTP remains available | [Scenario](../../demo-scenarios/errors/packet-id.jsonl) | Not specified |
| Initial state-query failure | Fail a required startup query | Producer ends while HTTP remains available | [Scenario](../../demo-scenarios/errors/initial-state-query.jsonl) | Not specified |
| Periodic state-query failure | Fail a later state query | Producer reconnects and recovers | [Scenario](../../demo-scenarios/errors/periodic-state-query.jsonl) | Not specified |
| Invalid attitude | With diagnostic logging enabled, inject a nonfinite primary value between valid data | Live attitude is suppressed until a fresh sample | [Scenario](../../demo-scenarios/errors/invalid-attitude.jsonl) | Not specified |
| Invalid optional value | Inject a nonfinite IAS value | Other fields remain available and IAS later recovers | [Scenario](../../demo-scenarios/errors/invalid-optional.jsonl) | Not specified |
| Native Bool | Exercise nonzero true and zero false for autopilot availability | Selected targets appear and then become unavailable | [Scenario](../../demo-scenarios/errors/native-bool.jsonl) | Not specified |
| Primary subscription failure | Required lifecycle subscription fails | Producer ends while HTTP remains available | [Scenario](../../demo-scenarios/errors/primary-subscribe.jsonl) | Not specified |
| Truncated packet | Deliberately malformed readable native bytes | Producer ends while HTTP remains available | [Scenario](../../demo-scenarios/errors/truncated-packet.jsonl) | Not specified |
| Invalid header size | An owned packet advertises an invalid header size | Producer ends while HTTP remains available | [Scenario](../../demo-scenarios/errors/invalid-header-size.jsonl) | Not specified |
| Truncated payload | An owned packet advertises more bytes than returned | Producer ends while HTTP remains available | [Scenario](../../demo-scenarios/errors/truncated-payload.jsonl) | Not specified |
| Invalid field count | An owned attitude packet advertises an incorrect field count | Producer ends while HTTP remains available | [Scenario](../../demo-scenarios/errors/invalid-count.jsonl) | Not specified |
| Close failure | Cleanup fails after required subscription failure | Producer ends with conservative native ownership | [Scenario](../../demo-scenarios/errors/close-failure.jsonl) | Not specified |
| Transport silence | Suppress data and control replies to exercise the watchdog | Source can recover after silence | [Scenario](../../demo-scenarios/errors/transport-silence.jsonl) | Not specified |
| Data-only gap | Suppress samples while state replies remain available | No watchdog reconnect; data later recovers | [Scenario](../../demo-scenarios/errors/data-gap.jsonl) | Not specified |

These are fictional inputs, not aircraft performance or simulator evidence. Terminal faults are separate from the flight tour.
