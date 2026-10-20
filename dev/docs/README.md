# Development documentation

## Tiling fork: start here

| Document | Purpose |
| --- | --- |
| [Current status](tiling-status.md) | Implemented behavior, latest evidence, remaining gates and tree precautions |
| [TTY cheat sheet](tty-preview.md) | Local preview launch, F-key bindings, exit and visual checks |
| [User guide](../../docs/tiling.md) | Available tiling behavior and configuration examples |
| [Parity roadmap](niri-tiling-parity-plan.md) | Full objective, single execution queue, dependencies and acceptance |
| [Reference inventory](niri-tiling-inventory.md) | Generated pinned symbol index; semantic audit remains partial |
| [Reference traces](niri-reference-traces.md) | Source-derived fixture method, assumptions and known differences |
| [History](tiling-history.md) | Earlier batch evidence, investigations and superseded next steps |
| [Initial proposal](scrolling-tiling-plan.md) | Historical limited-scope design; superseded by the parity roadmap |

Keep current facts in status, acceptance criteria in the roadmap, and detailed
past evidence in history. Update affected user instructions and preview keys when
behavior changes. Do not duplicate a competing task list or treat old percentages
and test counts as current measurements.

## Repository workflow

- [Git workflow](git-workflow.md): fork remotes, branch tracking, Compare and integration PRs.

- [Testing](testing.md): CI gates, fixture rules and generated references.
- [Caveats](caveats.md): architecture and known protocol/input constraints.
- [General TODO](todo.md): work outside the tiling roadmap.
- [Smithay API notes](smithay-api.md): locally verified API behavior.
- [Profiling](profiling.md) and [cross-distro builds](cross-distro-builds.md).
- [Reference config grammar](reference-config-format.md): config documentation workflow.

Read the root [AGENTS.md](../../AGENTS.md) and
[CONTRIBUTING.md](../../CONTRIBUTING.md) before implementation changes.
