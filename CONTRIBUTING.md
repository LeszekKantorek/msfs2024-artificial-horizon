# Contributing

## Environment setup

### 1. Install the tools

| Tool | Requirement / purpose |
| --- | --- |
| Windows x64 | Supported development and CI platform |
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
Follow [AGENTS.md](AGENTS.md) when starting agent work.
Use [task guidance](.context/task-guidance.md) to select procedures from project context and existing Markdown documentation.

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
The tested revision does not migrate the legacy `.context/sessions/` registry; retain it separately without treating its records as reviewed.

See [agent maintenance](.context/agent-maintenance.md) for reinstalls and integration updates.

### 4. Check the environment

1. Run the [automated checks](docs/testing.md#automated-checks), including the build and tests.
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
| Material architecture choice | Record the decision in `docs/adr/` under the [decision procedure](.context/architecture-decisions.md) |
| Tests | Check behavior independently of implementation details under [test design](.context/validation-and-review.md#select-evidence-by-failure-risk) |
| Runtime lifecycle | Preserve errors, source-loss handling, reconnects, and cancellation cleanup under [HTTP startup checks](docs/testing.md#http-startup-checks) |
| Telemetry | Update compatible fixtures and consumers with the [wire contract](docs/telemetry-contract.md) |
| Documentation | Distinguish planned behavior, implementation, and confirmed device acceptance under the [writing rules](.context/writing-style.md) |

Before delivery, apply the [validation scope matrix](docs/testing.md#validation-by-change-scope) and [evidence reuse rules](docs/testing.md#evidence-reuse).
Record results against the tested revision and environment, including observations and unavailable checks.
Use the [acceptance procedure](#acceptance-criteria-and-additional-verification) to track completion.
Before marking work complete, confirm that required checks pass and acceptance criteria have supporting evidence.
The [repository history](.context/repository-instructions.md#sources) records the origins of these requirements.

## Workflow

1. Select or create a GitHub issue with an outcome and acceptance criteria.
2. Check dependencies in the [roadmap](docs/roadmap.md).
3. Create a branch from current `main` named `change/<issue>-<topic>`, for example `change/20-responsive-pfd`.
4. Implement one reviewable outcome under the [change requirements](#change-requirements).
5. Update affected documentation, contracts, and fixtures.
6. Select and run checks through the [validation scope matrix](docs/testing.md#validation-by-change-scope).
7. Open a PR referencing the issue, changed behavior, and validation evidence. For new features, include an acceptance checklist for manual verification.

> Keep the checklist specific to the implemented feature. List only the main checks, without generic placeholders or detailed steps.

> Include `Closes #N` in a PR that implements the issue, including a draft PR.
> Record pending acceptance checks explicitly in the PR checklist; the closing keyword does not claim that those checks have passed.

### Acceptance criteria and additional verification

1. Give the issue an `Acceptance criteria` heading with Markdown task items.
2. Mark a criterion as checked when passing unit tests or automated validation confirm it.
3. Record the confirming tests or validation results in the issue or linked PR, including the tested commit and relevant environment.
4. Leave unconfirmed criteria unchecked and include their verification in the PR as unchecked items under `Additional verification`. Adapt the wording to the PR context when needed.
5. Describe how to verify the remaining items and what result confirms each one; update the issue and PR checkboxes after verification.

Draft PRs can retain unchecked verification; complete it before merge.
Use the [acceptance control reference](docs/pr-acceptance.md#checklist-scope) for parser rules and [status meanings](docs/pr-acceptance.md#understand-the-result) for blocking conditions.

### Promote agreed findings

Collect reusable implementation findings in [#9](https://github.com/LeszekKantorek/msfs2024-artificial-horizon/issues/9), referencing the originating task.

1. Review a finding after its originating task closes and agree on the follow-up scope.
2. Create the agreed issue with an outcome, acceptance criteria, and a link to the originating task.
3. Apply the requested labels and set the actual parent/sub-issue relationship when a parent is required.
4. Verify the saved labels and the parent shown in GitHub's Relationships section.

> A reference in the description or an epic task list does not establish a parent/sub-issue relationship.

## Project guidance

* Apply the [change requirements](#change-requirements) and [acceptance procedure](#acceptance-criteria-and-additional-verification) before submitting a change.
* Follow the [writing style and document responsibilities](.context/writing-style.md) for documentation, knowledge entries, code comments, commit messages, and GitHub issues, comments, and PRs.
* Follow [AGENTS.md](AGENTS.md) for agent work and [agent maintenance](.context/agent-maintenance.md) for skills and hooks.
