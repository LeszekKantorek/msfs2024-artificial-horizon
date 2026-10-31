# Contributing

## Environment setup

### 1. Install the tools

| Tool | Requirement / purpose |
| --- | --- |
| Windows x64 | Supported development, build, and runtime platform |
| Git | Clone the repository and manage changes |
| Visual Studio C++ Build Tools | MSVC linker and Windows SDK |
| Rust through rustup | Stable MSVC toolchain, rustfmt, and Clippy |
| Node.js 24 with npm/npx | Browser tests and skill installation |
| Python 3 on `PATH` | Session hooks use the `python` command |
| PowerShell | Run the commands below and Windows session hooks |
| Codex | Required only for the agent workflow and its skills/hooks |

> Demo development requires no simulator or SimConnect SDK. The web client requires no npm install or frontend build.

### 2. Prepare the checkout

Skip cloning if you already have a checkout.

```powershell
git clone https://github.com/LeszekKantorek/msfs2024-artificial-horizon.git
cd msfs2024-artificial-horizon
```

Run subsequent commands from the repository root.
Use the Windows x64 MSVC toolchain when installing rustup.

```powershell
rustup default stable
rustup component add rustfmt clippy --toolchain stable
```

The project does not pin a compiler version.

### 3. Prepare agent skills and hooks

The repository tests the skills from [context-skills PR #7](https://github.com/LeszekKantorek/context-skills/pull/7), branch `codex/context-skills-refresh`, pinned to commit `2598a6246b8649f7eaea68720201d9dbff1ddd84`:

| Skill | Purpose |
| --- | --- |
| `context-preserve` | Preserve decisions and findings with rationale and evidence |
| `context-review-sessions` | Review pending sessions through `context-preserve` |
| `context-map` | Connect tasks and questions to relevant sources |
| `context-maintain` | Improve knowledge coherence and organization |
| `context-diagnose` | Investigate agent mistakes and check corrections |
| `context-compile` | Turn established project procedures into skills |

If the installed copies are missing, install them with:

```powershell
npx --yes skills add https://github.com/LeszekKantorek/context-skills/tree/2598a6246b8649f7eaea68720201d9dbff1ddd84 --agent codex --skill context-preserve context-review-sessions context-map context-maintain context-diagnose context-compile --copy -y
```

The project also includes [procedural-skill-creator](https://github.com/LeszekKantorek/agent-skills/blob/main/skills/procedural-skill-creator/SKILL.md) for creating or revising executable skill procedures:

```powershell
npx --yes skills add LeszekKantorek/agent-skills --agent codex --skill procedural-skill-creator --copy -y
```

Choose the skill that fits the task; no skill sequence is required.
Follow [AGENTS.md](AGENTS.md#context-selection) for agent work and procedure selection.

#### Session hooks

The checkout also includes the [Codex integration from the tested revision](https://github.com/LeszekKantorek/context-skills/tree/2598a6246b8649f7eaea68720201d9dbff1ddd84/integrations/codex).
These files are separate from the skill installation command above:

| File | Purpose |
| --- | --- |
| [`.codex/hooks.json`](.codex/hooks.json) | Hook events and commands |
| [`.codex/hooks/context_register_session.py`](.codex/hooks/context_register_session.py) | Register session metadata without reading transcripts |

| Event | Action |
| --- | --- |
| `SessionStart` | Describe the six context skills so the agent can select one for the task |
| `Stop`, `Interrupt`, `PreCompact`, `SessionEnd` | Create or update one metadata row per session in Git-ignored `.context/sessions.log` |

To prepare the hooks:

1. Check the dependencies in PowerShell:

   ```powershell
   git --version
   python --version
   ```

2. Open the repository in the Codex CLI.
3. Open `/hooks` after installation or hook changes.
4. Review and trust the project definitions as described in the [hook documentation](https://learn.chatgpt.com/docs/hooks).

> Hooks register session metadata. They do not read transcripts or run reviews automatically.
> Use `context-review-sessions` with `context-preserve` to process pending records later.

The registry uses UTF-8 CSV with `session_date`, `session_id`, `path`, and `reviewed_date` columns.
Exclude `.context/sessions.log` and `.context/.sessions.lock` from commits and knowledge retrieval.
The session tools do not migrate or read `.context/sessions/`. Retain its metadata separately without treating its records as reviewed.

For reinstalls, preserve intentional local changes, use the pinned revision above, and update installed skills and `skills-lock.json` together.
Keep installer-generated hashes as source provenance and preserve unrelated hooks.

### 4. Check the environment

1. Run the [automated checks](docs/testing/automated.md#automated-checks), including the build and tests.
2. Start the demo:

   ```powershell
   cargo run --locked --bin msfs2024-artificial-horizon
   ```

3. Open [http://127.0.0.1:8080](http://127.0.0.1:8080).
4. Check that the attitude display appears.
5. Stop the server with `Ctrl+C`.

## Change requirements

Apply the [project constraints](AGENTS.md#project-constraints) and the requirements for the changed boundary:

| Changed boundary | Requirement / source |
| --- | --- |
| Dependencies | Commit `Cargo.lock` and exclude dependencies without distribution rights |
| Material architecture choice | Record the decision in `docs/adr/` under the [decision procedure](.agents/skills/project-decide-architecture/SKILL.md) |
| Tests | Check behavior independently of implementation details under [test design](docs/testing/evidence.md#select-evidence-by-failure-risk) |
| Runtime lifecycle | Preserve errors, source-loss handling, reconnects, and cancellation cleanup under [HTTP startup checks](docs/testing/runtime.md#http-startup-checks) |
| Telemetry | Update compatible fixtures and consumers with the [wire contract](docs/telemetry-contract.md) |
| Documentation | Distinguish planned behavior, implementation, and confirmed device acceptance under the [writing guidance](#writing-guidance) |

Before delivery, apply the [validation scope matrix](docs/testing/index.md#validation-by-change-scope) and [evidence reuse rules](docs/testing/index.md#evidence-reuse).
Local validation remains mandatory; GitHub Actions does not compile the application or run its test suites.
Record results in the PR, a related issue, or a durable linked validation record against the tested revision and environment, including observations and unavailable checks.
Use the [acceptance procedure](#acceptance-criteria-and-additional-verification) to track completion.
Before marking work complete, confirm that required checks pass and acceptance criteria have supporting evidence.

## Workflow

1. Establish the requested outcome and validation expectations. Consult a related issue when available.
2. Check dependencies in the [roadmap](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/9).
3. Create a branch from current `main` named `change/<topic>`.
4. Implement one reviewable outcome under the [change requirements](#change-requirements).
5. Update affected documentation, contracts, and fixtures.
6. Select and run checks through the [validation scope matrix](docs/testing/index.md#validation-by-change-scope).
7. Follow [PR preparation](#prepare-a-pull-request) to prepare the description or publish the requested PR.

### Prepare a pull request

Use [project-prepare-pr](.agents/skills/project-prepare-pr/SKILL.md) to prepare a local description or publish the requested PR.
The skill owns the preparation procedure; this document retains the contribution requirements and acceptance-tracking rules.
Describe what changed, why, and important design choices or consequences.

* Link a relevant issue when one exists. Use a closing keyword only when the PR completes that issue. Issue linkage and acceptance checklists are optional.
* Record applicable validation evidence in the PR, a related issue, or a durable linked record. No fixed test-status section or command list is required.

### Acceptance criteria and additional verification

1. Select required validation from the change scope.
2. Record confirming results in the PR, a related issue, or a durable linked record, including the tested commit and relevant environment.
3. If an issue uses an acceptance checklist, place its task items under `Acceptance criteria`. Mark them checked only after required validation confirms them.
4. Leave supplied unconfirmed criteria unchecked. Record remaining verification and expected results in the PR, related issue, or linked record.
5. Keep supplied checklists accurate. Unchecked criteria in a closing-linked issue and any visible unchecked PR task block the acceptance status.

Draft PRs can retain incomplete acceptance; complete it before merge.
A successful status does not replace required local validation.
Use the [acceptance control reference](docs/pr-acceptance.md#checklist-scope) for parser rules and [status meanings](docs/pr-acceptance.md#understand-the-result) for blocking conditions.

### Promote agreed findings

Collect reusable implementation findings in [#9](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/9), referencing the originating task.

1. Review a finding after its originating task closes and agree on the follow-up scope.
2. Create the agreed issue with an outcome, acceptance criteria, and a link to the originating task.
3. Apply the requested labels and set the actual parent/sub-issue relationship when a parent is required.
4. Verify the saved labels and the parent shown in GitHub's Relationships section.

Each follow-up issue must remain usable without the originating conversation or local notes:

* State the problem, relevant context, agreed scope, and exclusions.
* Link the canonical requirements rather than making the issue their only source.
* Include acceptance criteria and their validation method, with observable expected results.
* Preserve supporting findings and unresolved questions with their actual status.

> A reference in the description or an epic task list does not establish a parent/sub-issue relationship.

## Project guidance

* Apply the [change requirements](#change-requirements) and [acceptance procedure](#acceptance-criteria-and-additional-verification) before submitting a change.
* Follow the [writing guidance](#writing-guidance) for documentation, knowledge entries, code comments, commit messages, and GitHub issues, comments, and PRs.
* Follow [AGENTS.md](AGENTS.md) for agent work and the [setup instructions](#3-prepare-agent-skills-and-hooks) for skills and hooks.

## Writing guidance

* Apply [technical-writing-rules](.agents/skills/technical-writing-rules/SKILL.md) for English technical language and accuracy within its scope.
* Write concise, impersonal English with consistent terms across project text, including agent instructions and context entries.
* Describe results, decisions, and procedures without personal narration such as "the owner", "owner-assisted", "the user reported", or "the assistant verified". Use wording such as "The configuration selects..." or "The decision requires...".
* Retain technical ownership terms when they describe resource management or responsibilities.
* Use the [project terminology](docs/efi/index.md#terminology): EFI for the complete instrument, artificial horizon for attitude indication. Keep technical identifiers consistent with their consumers.
* Preserve requirements, units, limits, and uncertainty. Distinguish planned behavior, implementation, and confirmed acceptance.
* Keep supplied accepted plans local and verbatim, including their original language.
* Put feature requirements, presentation, implementation details, and tests on the owning feature page. Keep shared procedures in the testing guide.
* Link canonical public documentation rather than agent context. Check moved content, links, anchors, and Markdown paragraph boundaries.
* Use [project-write-content](.agents/skills/project-write-content/SKILL.md) for Markdown structure and examples, including when writing is part of implementation. Explicit project conventions take precedence over conflicting general defaults.

### Documentation maintenance

* Organize content for retrieval and understanding. Shortening text is not an independent goal.
* Describe current requirements and behavior directly. Omit implementation chronicles and accounts of document edits from current reference documentation.
* Retain the rationale for current decisions. Mark accepted planned behavior separately from existing implementation; unresolved claims remain unresolved.
* Before moving or shortening content, compare individual conditions, exceptions, values, and obligation levels. A requirement must not become a description or optional example.
* Before removing source text, verify where every requirement, constraint, and caveat remains in canonical documentation.
* Before restoring a historical requirement, check later accepted decisions and its current owner. Use Git history to resolve discrepancies.
* Identify incoming links before moving content and repair affected paths and anchors. Required link changes do not justify retaining content under the wrong owner.
* For test or acceptance claims, follow [historical evidence](docs/testing/evidence.md#historical-evidence-and-current-claims) and the [accepted-value policy](docs/testing/evidence.md#accepted-values-and-test-evidence). Preserve their conditions and scoped limits; do not copy these policies into writing guidance.
