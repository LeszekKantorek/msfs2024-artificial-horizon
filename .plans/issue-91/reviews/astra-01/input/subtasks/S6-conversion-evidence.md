# S6: Establish conversion evidence for native-input JSONL replay

## Issue body

Parent: Task #91 under Epic #9. Prerequisite: S2.

Execution: one GPT-6.1 Terra developer at medium. Use Sol Medium consultation for consequential gaps.
Read Task #91, Epic #9, this body, its first context comment, and prerequisite results.
Refresh issue artifacts and repository state before execution.
Follow the repository execution/reporting procedure; report results and gaps in the main conversation.
This is a local draft. It grants no implementation or publication authorization.

### Outcome

A bounded mapping result establishes which complete recorded sessions can become native-input Demo scenarios and which replay guarantees are supported.

### Scope

* Inspect current schema-4 production records and representative complete schema-5 records or their writer.
* Map raw fields, numeric bits, acquisition grouping, lifecycle, connection generations, timing, gaps, failures, and exclusions.
* Define explicit session selection, supported schemas, validation, loss reporting, and rejection conditions.
* Exclude converter implementation, recording-protocol changes, exact original SSE history, and estimator redesign.

### Context

* #91 accepted conversion comment and #125's old normalized-event mapping; refresh complete context and artifacts before relying on it.
* `runtime.rs` dispatch/raw/instrument logging and `journal.rs` timestamps/number representation.
* Native packet definitions, lifecycle, Instruments estimator, and S2's observations-mode contract.
* Representative real logs remain local and uncommitted.

### Implementation guidance

1. Establish actual input identities, completeness, schema, and update/source authority.
2. Build a field-by-field recoverability table: exact value, approximate timing, synthesized with rationale, or unavailable.
3. Preserve paired attitude grouping and independent optional observation times.
4. Distinguish raw attitude/instrument records from normalized publication snapshots.
5. Determine which lifecycle events and connection boundaries can be replayed without fabricated observations.
6. Explain the effects of new native receipt timestamps on age, expiry, and IAS acceleration.
7. Confirm or replace the proposed observation lateness policy using evidence permitted for that session.
8. Produce the final converter mapping, fidelity criteria, unsupported cases, and prerequisites for S7.

### Acceptance criteria

* [ ] Schema 4 mapping uses actual current writer fields and identifies exact numeric recovery versus timestamp approximation.
* [ ] Schema 5 support is established from actual representative records/writer, or remains explicitly unsupported pending that evidence.
* [ ] Mapping covers every currently consumed signal, grouping, control, gap, and unavailable case.
* [ ] Selected-session boundaries, ordering, completeness, supported schemas, and malformed-input rejection are unambiguous.
* [ ] Diagnostic metadata, local paths, identifiers, and diagnostic-only probes are excluded or sanitized under an explicit mapping.
* [ ] Unrecorded malformed bytes, wakeup sequences, original ages, and publication counts are not invented.
* [ ] Replay tolerances distinguish input order, timing, normalized values, availability, estimator output, and SSE observations.
* [ ] The result supplies enough resolved contract for Sol to prepare S7 without another open-ended investigation.
* [ ] Missing representative input or prohibited experiments leave affected support/fidelity criteria unconfirmed.

### Validation

Static inspection can establish source shapes and information-loss limits.
Use independent trace examples to assess mapping.
If runtime experiments become necessary, identify the exact uncertainty and obtain authorization appropriate to that later session.
The present session forbids tests, builds, and prototypes.
Keep logs and generated scenarios uncommitted.

## First comment: execution context

The old #125 mapping was established for normalized YAML events.
Its result remains useful evidence; it is not an unresolved old investigation.
The new question arises because playback now supplies native observations through an unchanged SimConnect consumer.

Current schema 4 records raw attitude and optional FLOAT64 values, including number classifications/bit patterns.
Decoded dispatch records preserve some lifecycle/control order.
The log does not preserve all original native bytes or every publication.

Schema 5 publication snapshots alone cannot recover raw acquisitions, exact original receipt times, cached observation identity, or every intermediate optional update.
Do not infer raw inputs by reversing normalization without labeling the reconstruction and establishing that it meets the required outcome.

The receiver timestamps each replayed dispatch with its new current time.
It therefore recomputes ages and the IAS estimator from the replay run.
No original timestamp field exists in the used native ABI.
A private timestamp backchannel or alternate Publisher path would change this architecture and needs a separate decision.

Both diagnostics and scenarios are JSONL, but their record schemas differ.
Keep offline conversion and one scenario reader.
Retain exact session selection and exclude local environment metadata.
If schema-5 raw evidence is unavailable, report the unsupported case instead of silently claiming old support.

The output of S6 is a durable mapping/fidelity contract and an execution handoff for S7.
It is not a completed converter.
