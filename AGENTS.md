# Agent instructions

## Before work

1. Use `context-apply` with [.context/index.md](.context/index.md) to select entries by the task's affected behavior.
2. Read the selected entries and only the source sections needed for the current change.
3. Use [task guidance](.context/task-guidance.md) when the route is unclear or the task crosses several areas.
4. Stop discovery when the expected result, applicable constraints, and validation method are clear.
5. Expand discovery when a dependency, conflict, missing fact, or changed scope requires it.

* A link is a reference, not an instruction to read every linked document recursively.
* Reuse relevant content already read in this conversation while it remains applicable and unchanged.
* Read README for startup or user-facing behavior, architecture for changed boundaries, and CONTRIBUTING for the applicable contribution stage.
* Apply every relevant route when a change crosses boundaries, including required validation and acceptance.
* When validating a change, select required checks from the [validation scope matrix](docs/testing.md#validation-by-change-scope).

## Project constraints

* Write project text in English and preserve the MIT license.
* Keep credentials, SDK binaries, local SDK paths, and generated output out of commits.
* Target Windows x64 MSVC for Rust work and keep default checks independent of the simulator.
* Preserve accepted scope and user changes, and keep supplied implementation plans local and verbatim.
* Report only validation actually performed or applicable recorded evidence, with missing checks stated explicitly.

## Project knowledge

| Skill | Use |
| --- | --- |
| `context-apply` | Retrieve and apply project knowledge |
| `context-gather` | Save session learning, review entries, and maintain the table-only index |
| `context-import-sessions` | Process selected saved sessions and invoke gathering |

* Follow each skill's instructions.
* Exclude `.context/sessions/` from knowledge retrieval.
* Use `context-import-sessions` to read selected session records.
