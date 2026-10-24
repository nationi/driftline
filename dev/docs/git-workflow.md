# Git workflow for this fork

## Remotes and branch policy

- `origin` (`https://github.com/nationi/driftline.git`) is the writable fork.
- `upstream` (`https://github.com/malbiruk/driftwm.git`) is the original project.
- Work on the fork's `main` branch. The former `nationi/development` and
  `rebrand/driftline` branches were merged and deleted; there is no persistent
  development branch.
- `main` contains fork-specific changes and is not an untouched upstream mirror.
  Never reset it to upstream or force-push it.

The [fork history](https://github.com/nationi/driftline/commits/main) is the
source of truth for current work. Compare the fork with upstream at
[upstream…fork](https://github.com/malbiruk/driftwm/compare/main...nationi:driftline:main).

## Local settings

These are repository-local preferences, not global account changes:

```bash
git config pull.ff only
git config push.default simple
git config remote.pushDefault origin
git config fetch.prune true
git config blame.ignoreRevsFile .git-blame-ignore-revs
```

`git pull` fast-forwards only. If it reports divergence, inspect the commits
and integrate the intended history explicitly; do not resolve it with an
automatic reset or force-push.

## Work on main

Start with a clean tree and fetch current refs:

```bash
git switch main
git status -sb
git fetch --all --prune
git pull --ff-only origin main
```

Commit focused changes directly to `main`, run the checks described in
[`testing.md`](testing.md), and push to the fork:

```bash
git push origin main
```

For a large or independently reviewable change, a temporary topic branch is
fine, but merge it back to `main` and delete it once integrated. Do not recreate
a permanent development or rebrand branch.

## Bring in upstream changes

Compare before merging; the fork intentionally diverges from upstream:

```bash
git fetch upstream
git log --oneline --left-right main...upstream/main
git diff --stat main...upstream/main
```

When an upstream update is useful, merge `upstream/main` into the fork's
`main`, resolve conflicts deliberately, run the full local CI, then push to
`origin main`. Preserve fork-specific tiling, branding, packaging removals and
workflow restrictions. Keep independent test-client additions from both sides;
do not resolve whole files with a blanket `ours` or `theirs` strategy.

Pull requests to the original repository, releases, repository-setting changes
and history rewrites are separate decisions. This workflow does not authorize
them.
