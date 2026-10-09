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

The repository includes these skills from [LeszekKantorek/context-skills](https://github.com/LeszekKantorek/context-skills):

| Skill | Purpose |
| --- | --- |
| `context-apply` | Read and apply project knowledge before work |
| `context-gather` | Save learning and maintain knowledge entries and their index |
| `context-import-sessions` | Read selected saved sessions and pass them to gathering |

If the installed copies are missing, install them with:

```powershell
npx --yes skills add LeszekKantorek/context-skills --agent codex --skill context-apply context-gather context-import-sessions --copy -y
```

Follow [AGENTS.md](AGENTS.md) when starting agent work.

#### Session hooks

The checkout also includes the [context-skills Codex integration](https://github.com/LeszekKantorek/context-skills/tree/main/integrations/codex).
These files are separate from the skill installation command above:

| File | Purpose |
| --- | --- |
| [`.codex/hooks.json`](.codex/hooks.json) | Hook events and commands |
| [`.codex/hooks/context_register_session.py`](.codex/hooks/context_register_session.py) | Register session metadata without reading transcripts |

| Event | Action |
| --- | --- |
| `SessionStart` | Remind the agent to apply project knowledge and use the context skills |
| `Stop`, `Interrupt`, `PreCompact`, `SessionEnd` | Create or update one metadata record per session in Git-ignored `.context/sessions/` |

To prepare the hooks:

1. Check the dependencies in PowerShell:

   ```powershell
   git --version
   python --version
   ```

2. Open the repository in the Codex CLI.
3. Open `/hooks` after installation or hook changes.
4. Review and trust the project definitions as described in the [hook documentation](https://learn.chatgpt.com/docs/hooks).

> Hooks register session metadata. They do not read transcripts or run `context-gather` automatically.
> Use `context-import-sessions` to process selected records later.

See [agent maintenance](.context/agent-maintenance.md) for reinstalls and integration updates.

### 4. Check the environment

1. Run the [automated checks](docs/testing.md#automated-checks), including the build and current SDK-free SimConnect checks.
2. Start the demo:

   ```powershell
   cargo run --locked --bin main
   ```

3. Open [http://127.0.0.1:8080](http://127.0.0.1:8080).
4. Check that DEMO and the attitude display appear.
5. Stop the server with `Ctrl+C`.

## Workflow

1. Select or create a GitHub issue with an outcome and acceptance criteria.
2. Check dependencies in the [roadmap](docs/roadmap.md).
3. Create a branch from current `main` named `change/<issue>-<topic>`, for example `change/20-responsive-pfd`.
4. Implement one reviewable outcome.
5. Update affected documentation, contracts, and fixtures.
6. Run the relevant [checks](docs/testing.md#automated-checks).
7. Open a PR referencing the issue, changed behavior, and validation evidence. For new features, include an acceptance checklist for manual verification.

Keep the checklist specific to the implemented feature. List only the main checks, without generic placeholders or detailed steps.

> Use `Closes #N` only when evidence satisfies all acceptance criteria.
> Keep unverified hardware acceptance open or create an explicit linked follow-up.

## Project guidance

* Check [repository rules and acceptance criteria](.context/repository-instructions.md#repository-rules) before submitting a change.
* Follow the [writing style and document responsibilities](.context/writing-style.md) for documentation, knowledge entries, code comments, commit messages, and GitHub issues, comments, and PRs.
* Run the relevant [automated checks](docs/testing.md#automated-checks).
* Follow [AGENTS.md](AGENTS.md) for agent work and [agent maintenance](.context/agent-maintenance.md) for skills and hooks.
