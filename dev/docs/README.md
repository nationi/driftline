# Development documentation

## Tiling fork: start here

| Document | Purpose |
| --- | --- |
| [Current status](tiling-status.md) | Implemented behavior, validation evidence and remaining manual checks |
| [TTY cheat sheet](tty-preview.md) | Local preview launch, F-key bindings, exit and visual checks |
| [User guide](../../docs/tiling.md) | Available tiling behavior and configuration examples |
| [Practical roadmap](scrolling-tiling-roadmap.md) | Focused product scope, delivery sequence and acceptance |

Keep current behavior and validation evidence in status, acceptance criteria in
the roadmap. Update affected user instructions and preview keys when behavior
changes. Niri remains an implementation reference, not a compatibility target.

## Repository workflow

- [Git workflow](git-workflow.md): fork remotes, work on `main`, and integrate upstream updates.
- [Testing](testing.md): CI gates, fixture rules and generated references.
- [Caveats](caveats.md): architecture and known protocol/input constraints.
- [Smithay API notes](smithay-api.md): locally verified API behavior.
- [Profiling](profiling.md) and [cross-distro builds](cross-distro-builds.md).
- [Reference config grammar](reference-config-format.md): config documentation workflow.

Read the root [AGENTS.md](../../AGENTS.md) and
[CONTRIBUTING.md](../../CONTRIBUTING.md) before implementation changes.
