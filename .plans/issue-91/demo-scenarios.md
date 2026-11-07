# Demo scenario catalog

Status: Local documentation draft. Intended repository owner: `docs/sources/demo-scenarios.md`.
The linked JSONL files are planning drafts, not executed scenarios.
During implementation, move scenario links to the committed `demo-scenarios/` directory.

## Purpose and ownership

Use this catalog to select a scenario by its purpose and expected application behavior.
JSONL owns signal values, durations, event order, and fault parameters.
Each scenario may provide any subset of signals, lifecycle events only, or no signal values.
Missing signals stay silent; complete native-field coverage belongs to the suite rather than each file.
Do not repeat the timeline, speed tables, interpolation endpoints, or per-event assertions in this document.

Playback returns to the first step after the last step while the clock keeps increasing.
Pause and flight state follow explicit scenario records; no state is restored automatically between passes.
Terminal source failures can prevent further playback. They do not define a normal end-of-file waiting mode.

| Scenario | Main assumptions and coverage | Expected completion |
| --- | --- | --- |
| [Flight tour](scenarios/flight-tour.jsonl) | Fictional runway, takeoff, climb, slowing and stall/recovery narrative, turns, and landing. Includes normal/active pause and recoverable simulator availability changes | Completes each narrative pass and repeats until shutdown; no terminal native failure intended |
| [Simulator lifecycle](scenarios/simulator-lifecycle.jsonl) | Initial unavailable connection, active/inactive flight, both pause modes, flight change, simulator stop, and reconnect. Distinguishes source connection from flight readiness | Recovers where scheduled and repeats the steps until shutdown |
| [Independent availability](scenarios/availability-gaps.jsonl) | Equal fresh readings, independent signal silence, primary-sample expiry, target gating, and recovery | Other independent data remains usable; no fatal failure intended |
| [Optional definition failure](scenarios/errors/optional-definition.jsonl) | A failed optional field registration must not terminate attitude acquisition or HTTP | Producer continues with that field unavailable |
| [Primary subscription failure](scenarios/errors/primary-subscribe.jsonl) | A required lifecycle subscription fails during configuration | Producer ends; HTTP remains available with unavailable source data |
| [Truncated native packet](scenarios/errors/truncated-packet.jsonl) | Owned readable bytes contain an invalid native message shape | Decoder rejects it; producer ends; HTTP remains available |
| [Close failure](scenarios/errors/close-failure.jsonl) | Connection loss reaches a native cleanup failure | Producer ends; native mapping ownership remains conservative until process exit |
| [Transport silence](scenarios/errors/transport-silence.jsonl) | Total connection silence also suppresses system-state replies, allowing the existing watchdog to run | Source becomes stale, retries, and can recover when silence ends |

The existing default's mapped instrument boundaries remain a separate regression scenario.
Its planned owner remains repository-root `demo_scenario.jsonl`.
The flight tour is an additional explicit selection; this revision does not silently replace the agreed default.

Additional focused fault files belong beside these scenarios when their error family is implemented.
A scenario ending its producer must be isolated from the flight tour and from unrelated terminal fault cases.

## Interpretation limits

* Values describe a fictional demonstration, not an aircraft performance model or validated flight dynamics.
* Rotation uses the existing minimum-rotation reference. The current input surface supplies no Vx reference.
* The climb-speed phase can illustrate that part of the story without adding a Vx marker or private SimVar.
* The stall/recovery narrative uses speed, attitude, altitude, and VSI. It does not test a stall-warning signal or aerodynamic stall detection.
* Runway and touchdown are narrative context. The current receiver has no ground-contact or runway-position input.
* Simulator absence/QUIT is different from Sim-running false, a missing loaded flight, and a paused flight.
* Both pause modes produce unavailable telemetry under the current receiver policy, using their distinct native pause flags.
* That publication policy does not imply native packet silence. The current assumption repeats frozen values while messages continue; verify normal/active pause delivery during implementation and align authored pause holds accordingly.
* Scenario time continues during pauses. Recovery uses current scripted values and new observations rather than replaying a missed frame backlog.
* Recoverable connection unavailability belongs in a normal narrative. Malformed packets and terminal API failures use dedicated files.

## Use scenarios in tests

Use the flight tour for application integration and visible instrument progression.
Use focused scenarios for lifecycle, availability, native errors, and cleanup.
Retain unit and boundary fixtures for exact calculations and display capabilities not available through native acquisition.

Tests define independent expected checkpoints and outcomes.
Do not derive expected values from the scenario reader, scheduler, or source file under test.
Keep test-specific timestamps and numeric assertions in fixtures/tests, outside this catalog.

Loading errors, missing exports, logging destination failures, and application infrastructure failures need launch or test-harness conditions.
They cannot all be represented by an instruction inside a DLL that has not loaded.
Each test records whether failure affects a field, connection, producer, client, or entire application.
