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

Prepare the toolchain under these requirements:

* Run subsequent commands from the repository root.
* Use the Windows x64 MSVC toolchain when installing rustup.

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

Select agent guidance under these requirements:

* Choose the skill that fits the Task. No fixed skill sequence is required.
* Follow [AGENTS.md](AGENTS.md#context-selection) for agent work and procedure selection.

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

Apply these hook and session-review constraints:

* Treat hook output as session metadata. Hooks do not read transcripts or run reviews automatically.
* Use `context-review-sessions` with `context-preserve` to process pending records later.

Maintain the session registry and installed integration under these requirements:

* Preserve the UTF-8 CSV columns `session_date`, `session_id`, `path`, and `reviewed_date` in the registry.
* Exclude `.context/sessions.log` and `.context/.sessions.lock` from commits and knowledge retrieval.
* Retain `.context/sessions/` metadata separately without treating it as reviewed. The session tools do not migrate or read that directory.
* For reinstalls, preserve intentional local changes and use the pinned revision in [agent setup](#3-prepare-agent-skills-and-hooks).
* Update installed skills and `skills-lock.json` together. Preserve installer-generated hashes as source provenance and keep unrelated hooks.

### 4. Check the environment

Check the environment before starting development:

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

Before delivery, satisfy these validation and evidence requirements:

* Apply the [validation scope matrix](docs/testing/index.md#validation-by-change-scope) and [evidence reuse rules](docs/testing/index.md#evidence-reuse).
* Run required local validation. GitHub Actions does not compile the application or run its test suites.
* Record the tested revision, environment, observations, results, and unavailable checks in the PR, related issue, or durable linked validation record.
* Track completion through the [acceptance procedure](#acceptance-criteria-and-additional-verification).
* Confirm that required checks pass and acceptance criteria have supporting evidence before marking work complete.

## Workflow

Follow the [work organization model](docs/work-organization.md) for issue levels, responsibilities, and iterative planning when carrying out this workflow:

1. Read the Task, its Work Item, and the relevant Initiative. For work without an issue, establish the requested outcome and validation expectations directly.
2. Check prerequisites and current context against the relevant source sections and code. Use accepted decisions and report consequential gaps before dependent work.
3. Create a branch from current `main` named `change/<topic>`.
4. Deliver one reviewable outcome under the [change requirements](#change-requirements). A diagnostic, research, or documentation Task does not require application code changes.
5. Update affected documentation, contracts, and fixtures.
6. Select and run checks through the [validation scope matrix](docs/testing/index.md#validation-by-change-scope).
7. Follow [PR preparation](#prepare-a-pull-request) to prepare the description or publish the requested PR.

### Prepare a pull request

Use [project-prepare-pr](.agents/skills/project-prepare-pr/SKILL.md) for local descriptions or authorized publication under these contribution requirements:

* Describe what changed, why, and important design choices or consequences.
* Link the Task when one exists and reference its Work Item for context. Use a closing keyword only when the PR completes that issue's criteria.
* A Task can use several PRs or no PR. Do not close its Work Item or Initiative solely because this PR completes a Task.
* Issue linkage and acceptance checklists remain optional for work without an issue. Task instructions must still define observable acceptance criteria.
* Record applicable validation evidence in the PR, a related issue, or a durable linked record. No fixed test-status section or command list is required.

### Acceptance criteria and additional verification

Verify and record acceptance in this order:

1. Select required validation from the change scope.
2. Record confirming results in the PR, a related issue, or a durable linked record, including the tested commit and relevant environment.
3. If an issue uses an acceptance checklist, place its task items under `Acceptance criteria`. Mark them checked only after required validation confirms them.
4. Leave supplied unconfirmed criteria unchecked. Record remaining verification and expected results in the PR, related issue, or linked record.
5. Keep supplied checklists accurate. Unchecked criteria in a closing-linked issue and any visible unchecked PR task block the acceptance status.

Apply the [completion rules](docs/work-organization.md#completion-and-evidence) at each issue level:

* Close a Task only after its outcome and required verification are supported by evidence. A diagnostic result must distinguish confirmed findings from unresolved causes.
* Evaluate each Work Item and Initiative against its own result and criteria. Completed children alone do not establish acceptance.
* Keep a Work Item open for further planning when diagnostic results identify additional work or leave its required result unconfirmed.
* Change issue status and publish evidence only within authorized GitHub work.

Apply these merge and status constraints:

* Draft PRs can retain incomplete acceptance. Complete required acceptance before merge.
* Perform required local validation even when the acceptance status succeeds.
* Use the [acceptance control reference](docs/pr-acceptance.md#checklist-scope) for parser rules and [status meanings](docs/pr-acceptance.md#understand-the-result) for blocking conditions.

### Promote agreed findings

Use confirmed findings to continue Work Item planning or define an agreed follow-up through these steps:

1. Review the evidence, distinguish established findings from hypotheses, and define the authorized follow-up scope.
   Diagnostic results can support further planning before the originating Task closes.
2. Preserve durable requirements and reusable findings in their documentation owners. Link the originating Task and relevant results in the follow-up context.
3. Prepare a Work Item or Task at the appropriate level. Publish it only when authorized; otherwise, return a complete draft.
4. For authorized publication, verify label availability, apply the level label, and establish and verify the actual parent/sub-issue relationship.

Each follow-up issue must remain usable without the originating conversation or local notes:

* State the problem, relevant context, agreed scope, and exclusions.
* Link the canonical requirements rather than making the issue their only source.
* Include acceptance criteria and their validation method, with observable expected results.
* Preserve supporting findings and unresolved questions with their actual status.

> A reference in the description or a checklist does not establish a parent/sub-issue relationship.

## Project guidance

Apply these contribution and documentation requirements:

* Apply the [change requirements](#change-requirements) and [acceptance procedure](#acceptance-criteria-and-additional-verification) before submitting a change.
* Follow the [writing guidance](#writing-guidance) for documentation, knowledge entries, code comments, commit messages, and GitHub issues, comments, and PRs.
* Follow [AGENTS.md](AGENTS.md) for agent work and the [setup instructions](#3-prepare-agent-skills-and-hooks) for skills and hooks.

## Writing guidance

Write project text under these requirements:

* Apply [technical-writing-rules](.agents/skills/technical-writing-rules/SKILL.md) for English technical language and accuracy within its scope.
* Write concise, impersonal English with consistent terms across project text, including agent instructions and context entries.
* Describe results, decisions, and procedures without personal narration such as "the owner", "owner-assisted", "the user reported", or "the assistant verified". Use wording such as "The configuration selects..." or "The decision requires...".
* Retain technical ownership terms when they describe resource management or responsibilities.
* Use the [project terminology](docs/efi/index.md#terminology): EFI for the complete instrument, artificial horizon for attitude indication. Keep technical identifiers consistent with their consumers.
* Preserve requirements, units, limits, and uncertainty. Distinguish planned behavior, implementation, and confirmed acceptance.
* Keep supplied accepted plan originals local and verbatim, including their original language. Prepare Task instructions in English while preserving accepted requirements and decisions.
* Put feature requirements, presentation, implementation details, and tests on the owning feature page. Keep shared procedures in the testing guide.
* Link canonical public documentation rather than agent context. Check moved content, links, anchors, and Markdown paragraph boundaries.
* Use [project-write-content](.agents/skills/project-write-content/SKILL.md) for Markdown structure and examples, including when writing is part of implementation. Explicit project conventions take precedence over conflicting general defaults.

### Documentation maintenance

Maintain documentation under these requirements:

* Organize content for retrieval and understanding. Shortening text is not an independent goal.
* Describe current requirements and behavior directly. Omit implementation chronicles and accounts of document edits from current reference documentation.
* Retain the rationale for current decisions. Mark accepted planned behavior separately from existing implementation; unresolved claims remain unresolved.
* Before moving or shortening content, compare individual conditions, exceptions, values, and obligation levels. A requirement must not become a description or optional example.
* Before removing source text, verify where every requirement, constraint, and caveat remains in canonical documentation.
* Before restoring a historical requirement, check later accepted decisions and its current owner. Use Git history to resolve discrepancies.
* Identify incoming links before moving content and repair affected paths and anchors. Required link changes do not justify retaining content under the wrong owner.
* For test or acceptance claims, follow [historical evidence](docs/testing/evidence.md#historical-evidence-and-current-claims) and the [accepted-value policy](docs/testing/evidence.md#accepted-values-and-test-evidence). Preserve their conditions and scoped limits; do not copy these policies into writing guidance.
