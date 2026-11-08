# Demo scenario catalog

The suite contains a narrated flight, focused availability and lifecycle inputs, and selected fault scenarios. The [Demo contract](demo.md#native-demo-playback) defines playback; scenario files own values, durations, event order, and fault parameters. Each scenario can supply a subset of signals or only events. The [native error contract](simconnect.md#dispatch-and-error-limits) owns error outcomes.

| Scenario | Purpose and assumptions | Expected completion | File | Visual regression |
| --- | --- | --- | --- | --- |
| Baseline boundaries | Preserve applicable current native-input cycles and independent boundary checks | Repeats until shutdown | [Baseline scenario](../../demo-scenarios/demo-scenario.jsonl) | Not specified |
| Flight tour | Fictional flight progression, both pause modes, and recoverable connection changes; no flight dynamics or ground-contact model | Repeats until shutdown | [Flight tour](../../demo-scenarios/flight-tour.jsonl) | No. Only unit tests and manual acceptance testing |
| Simulator lifecycle | Connection availability, inactive flight, pause, flight change, and reconnect | Recovers where scheduled and repeats | [Simulator lifecycle](../../demo-scenarios/simulator-lifecycle.jsonl) | Not specified |
| Independent availability | Equal fresh samples, signal gaps, target gating, and recovery | Independent signals remain usable | [Availability gaps](../../demo-scenarios/availability-gaps.jsonl) | Not specified |
| IAS only | Sparse native IAS without attitude | Stays unavailable for live display | [IAS only](../../demo-scenarios/ias-only.jsonl) | Not specified |
| Event only | Lifecycle events without instrument tracks | Repeats until shutdown | [Event only](../../demo-scenarios/event-only.jsonl) | Not specified |
| No signals | Initial lifecycle without instrument tracks | Repeats until shutdown | [No signals](../../demo-scenarios/no-signal.jsonl) | Not specified |
| Startup retry | First Open fails; the following attempt recovers | Continues with live data and HTTP available | [Scenario](../../demo-scenarios/errors/startup-retry.jsonl) | Not specified |
| Optional definition failure | Isolate one failed optional registration | Producer continues with that field unavailable | [Scenario](../../demo-scenarios/errors/optional-definition.jsonl) | Not specified |
| Primary subscription failure | Required lifecycle subscription fails | Producer ends while HTTP remains available | [Scenario](../../demo-scenarios/errors/primary-subscribe.jsonl) | Not specified |
| Truncated packet | Deliberately malformed readable native bytes | Producer ends while HTTP remains available | [Scenario](../../demo-scenarios/errors/truncated-packet.jsonl) | Not specified |
| Close failure | Cleanup fails after connection loss | Producer ends with conservative native ownership | [Native errors](simconnect.md#dispatch-and-error-limits) | Not specified |
| Transport silence | Suppress data and control replies to exercise the watchdog | Source can recover after silence | [Demo faults](demo.md#lifecycle-and-faults) | Not specified |

These are fictional inputs, not aircraft performance or simulator evidence. Terminal faults are separate from the flight tour. Remaining files and links will be added with their corresponding implementations. Uncorrelated exception, failed Close, and transport-silence scenarios are deferred.
