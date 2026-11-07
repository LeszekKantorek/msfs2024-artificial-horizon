# Demo scenario catalog

Selected fault scenarios are available; other listed scenarios are planned. The [Demo contract](demo.md#native-demo-playback) defines playback; scenario files will own values, durations, event order, and fault parameters. Each scenario can supply a subset of signals or only events. The [native error contract](simconnect.md#dispatch-and-error-limits) owns error outcomes.

| Scenario | Purpose and assumptions | Expected completion | File or planned owner |
| --- | --- | --- | --- |
| Default boundaries | Preserve applicable current native-input cycles and independent boundary checks | Repeats until shutdown | [Demo contract](demo.md#native-demo-playback) |
| Flight tour | Fictional flight progression, both pause modes, and recoverable connection changes; no flight dynamics or ground-contact model | Repeats until shutdown | [Demo lifecycle](demo.md#lifecycle-and-faults) |
| Simulator lifecycle | Connection availability, inactive flight, pause, flight change, and reconnect | Recovers where scheduled and repeats | [Demo lifecycle](demo.md#lifecycle-and-faults) |
| Independent availability | Equal fresh samples, signal gaps, target gating, and recovery | Independent signals remain usable | [Demo tracks](demo.md#scripted-signal-tracks) |
| Startup retry | First Open fails; the following attempt recovers | Continues with live data and HTTP available | [Scenario](../../demo-scenarios/errors/startup-retry.jsonl) |
| Optional definition failure | Isolate one failed optional registration | Producer continues with that field unavailable | [Scenario](../../demo-scenarios/errors/optional-definition.jsonl) |
| Primary subscription failure | Required lifecycle subscription fails | Producer ends while HTTP remains available | [Scenario](../../demo-scenarios/errors/primary-subscribe.jsonl) |
| Truncated packet | Deliberately malformed readable native bytes | Producer ends while HTTP remains available | [Scenario](../../demo-scenarios/errors/truncated-packet.jsonl) |
| Close failure | Cleanup fails after connection loss | Producer ends with conservative native ownership | [Native errors](simconnect.md#dispatch-and-error-limits) |
| Transport silence | Suppress data and control replies to exercise the watchdog | Source can recover after silence | [Demo faults](demo.md#lifecycle-and-faults) |

These are fictional inputs, not aircraft performance or simulator evidence. Terminal faults are separate from the flight tour. Remaining files and links will be added with their corresponding implementations. Uncorrelated exception, failed Close, and transport-silence scenarios are deferred.
