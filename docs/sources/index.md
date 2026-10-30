# Source providers

Providers acquire or generate flight data, normalize it into validated telemetry, and publish it through a caller-owned channel.
Demo and SimConnect share the same downstream HTTP/SSE and EFI interfaces.

## Select a provider

| Provider | Use when | Reference |
| --- | --- | --- |
| Demo | Developing or validating deterministic behavior without a simulator | [Demo API, scenarios, and validation](demo.md) |
| SimConnect | Receiving available flight instruments from MSFS2024 or collecting native diagnostics | [SimConnect configuration, implementation, and evidence limits](simconnect.md) |

The CLI defaults to Demo. Library application calls select `application::SourceSelection` explicitly.
`Source` identifies `demo` or `simconnect` in telemetry and remains fixed for the channel lifetime.
Selection is runtime configuration, not a Cargo feature or a separate package.

## Shared responsibilities

* Keep SDK types and native unit/sign conventions inside the provider boundary.
* Publish validated normalized values using the [telemetry model](../telemetry-contract.md#model-boundary).
* Emit live data only for fresh usable acquisition. Publish explicit unavailable states for source loss or invalid input.
* Preserve the acquisition timestamp when publishing cached optional observations. Cached values cannot renew attitude age.
* Evaluate freshness independently of callbacks. Unavailable snapshots contain no live attitude.
* Keep optional indications independent and preserve attitude-only publication.
* Never replace unavailable data with zero or Demo values.
* Accept caller-owned publication and shutdown without reading application arguments or HTTP configuration.
* Create no hidden channel, telemetry history queue, or HTTP server.

Application composition creates the channel, binds HTTP, and then starts the selected provider.
The [application lifecycle](../architecture.md#server-and-browser-ownership) owns failure supervision and shutdown coordination.
A source failure never selects Demo automatically.

The [library telemetry interface](../architecture.md#library-telemetry-interface) defines publication and subscription semantics.
The [wire contract](../telemetry-contract.md) defines units, validity, freshness, and compatibility.
Use [runtime validation](../testing/runtime.md#telemetry-and-demo-checks) for shared behavior and each provider's guide for source-specific checks and evidence limits.
