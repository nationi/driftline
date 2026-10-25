# Development documentation

Read [CONTRIBUTING.md](../../CONTRIBUTING.md) and
[AGENTS.md](../../AGENTS.md) before making implementation changes.

| Guide | Purpose |
| --- | --- |
| [Architecture caveats](caveats.md) | Stage ownership, protocol and input constraints |
| [Testing](testing.md) | Test map, CI, generated docs and hardware acceptance |
| [Tiling status](tiling-status.md) | Scope, current validation and remaining checks |
| [Smithay API notes](smithay-api.md) | Verified signatures and integration behavior |
| [Profiling](profiling.md) | Tracy capture and analysis |
| [Config reference format](reference-config-format.md) | Grammar used by config tests and documentation generation |

User-facing behavior and shortcuts belong in [docs/](../../docs/), starting with
[scrolling tiling](../../docs/tiling.md), rather than in development plans.
