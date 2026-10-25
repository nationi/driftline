# Contributing

Thanks for your interest in driftline. Issues and pull requests are welcome.

## Before you start

**Open an issue first for non-trivial changes.** Anything beyond a quick fix — features, refactors, multi-file changes — should start as an issue so we can align on the approach before you invest time. Small fixes can go straight to PR.

**Read the internal docs in [`dev/docs/`](dev/docs/) before touching code** — architectural rules ([`caveats.md`](dev/docs/caveats.md)) and testing conventions ([`testing.md`](dev/docs/testing.md)) especially.

**Using an AI coding agent?** Read [`AGENTS.md`](AGENTS.md) for project instructions.

## Pull requests

**Keep PRs small and focused on one concern.** One PR = one logical change. If your description says "this does X and Y", that's two PRs.

When changes bundle multiple concerns, merging becomes all-or-nothing — if I like parts but not others, we lose a round trip asking you to split. Split up front and each piece lands (or doesn't) independently.

**CI must pass.** The authoritative checks are listed in
[`dev/scripts/ci-local`](dev/scripts/ci-local) and run by
[`.github/workflows/ci.yml`](.github/workflows/ci.yml). Run the relevant stages
locally before opening a PR.

**Only state belongs in `src/state/mod.rs`** — the `Driftline` fields, the types they're made of, and the routines that must enumerate every field. A new `impl Driftline` method goes in the sibling module for the subsystem it serves.

## Blaming through reformats

Bulk reformats and pure file-motion commits are listed in [`.git-blame-ignore-revs`](.git-blame-ignore-revs), so `git blame` reaches the commit that actually wrote a line. GitHub applies it automatically; locally:

```bash
git config blame.ignoreRevsFile .git-blame-ignore-revs
```

## Reporting bugs

Include:

- What you expected vs what happened
- Steps to reproduce
- Distro, GPU, nested vs TTY, driftline version
- `RUST_LOG=debug` logs if relevant

## Fork workflow

This fork is maintained at [nationi/driftline](https://github.com/nationi/driftline).
Use its issues and pull requests for fork-specific changes.
`origin` points to the fork; `upstream` points to the original
[malbiruk/driftwm](https://github.com/malbiruk/driftwm) project.

The integration branch is `main`. Temporary topic branches are welcome for pull
requests. Before integrating work, fetch current refs and inspect any divergence;
use fast-forward-only pulls and never reset or force-push `main`. Upstream merges
must preserve the fork's tiling, branding and packaging decisions and pass CI.
