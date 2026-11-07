# Planned Demo scenario catalog

These scenarios are planned and are not yet delivered. The [Demo contract](demo.md#native-demo-playback) defines playback; scenario files will own values, durations, event order, and fault parameters. Each scenario can supply a subset of signals or only events. The [native error contract](simconnect.md#dispatch-and-error-limits) owns error outcomes.

| Scenario | Purpose and assumptions | Expected completion | Planned owner |
| --- | --- | --- | --- |
| Default boundaries | Preserve applicable current native-input cycles and independent boundary checks | Repeats until shutdown | [Demo contract](demo.md#native-demo-playback) |
| Flight tour | Fictional flight progression, both pause modes, and recoverable connection changes; no flight dynamics or ground-contact model | Repeats until shutdown | [Demo lifecycle](demo.md#lifecycle-and-faults) |
| Simulator lifecycle | Connection availability, inactive flight, pause, flight change, and reconnect | Recovers where scheduled and repeats | [Demo lifecycle](demo.md#lifecycle-and-faults) |
| Independent availability | Equal fresh samples, signal gaps, target gating, and recovery | Independent signals remain usable | [Demo tracks](demo.md#scripted-signal-tracks) |
| Optional definition failure | Isolate one failed optional registration | Producer continues with that field unavailable | [Native errors](simconnect.md#dispatch-and-error-limits) |
| Primary subscription failure | Required lifecycle subscription fails | Producer ends while HTTP remains available | [Native errors](simconnect.md#dispatch-and-error-limits) |
| Truncated packet | Deliberately malformed readable native bytes | Producer ends while HTTP remains available | [Native errors](simconnect.md#dispatch-and-error-limits) |
| Close failure | Cleanup fails after connection loss | Producer ends with conservative native ownership | [Native errors](simconnect.md#dispatch-and-error-limits) |
| Transport silence | Suppress data and control replies to exercise the watchdog | Source can recover after silence | [Demo faults](demo.md#lifecycle-and-faults) |

These are fictional inputs, not aircraft performance or simulator evidence. Terminal faults are separate from the flight tour. Files and their links will be added with the corresponding implementations.
