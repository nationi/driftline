# Git workflow for this fork

## Branches and remotes

- `origin` is `https://github.com/nationi/driftwm.git` — the writable fork.
- `upstream` is `https://github.com/malbiruk/driftwm.git` — the reference source.
- `main` is the fork's consolidated branch, not an untouched upstream mirror.
- `nationi/development` is the persistent development branch. Do not delete it
  when merging its pull requests. Both local branches track their `origin` peers.

Compare the fork's branches at
[main…development](https://github.com/nationi/driftwm/compare/main...nationi/development).
Compare upstream with the consolidated fork at
[upstream…fork](https://github.com/malbiruk/driftwm/compare/main...nationi:driftwm:main).
Being ahead of upstream is expected for a feature fork; it is not a reason to
reset or force-push the branch.

## Local settings

These are repository-local preferences, not global account changes:

```bash
git config pull.ff only
git config push.default simple
git config remote.pushDefault origin
git config fetch.prune true
git config blame.ignoreRevsFile .git-blame-ignore-revs
```

`git pull` fast-forwards only. If it reports divergence, inspect and merge the
intended branch explicitly; do not fix it with an automatic reset or force push.

## Receive changes

Start with a clean working tree and fetch current refs:

```bash
git fetch --all --prune
git switch nationi/development
git pull --ff-only
git log --oneline --left-right origin/main...HEAD
git log --oneline --left-right upstream/main...HEAD
```

Merge `origin/main` first when it has changes missing from development. Then
merge `upstream/main` if needed. Preserve the fork's layout implementation,
packaging removals and workflow restrictions when resolving conflicts. Keep
both sides of independent test-client additions; do not resolve all files with
one blanket `ours` or `theirs` strategy.

Run the appropriate targeted checks and full `dev/scripts/ci-local` before
publishing an integration change. If canonical config documentation changed,
follow [the generated-reference workflow](testing.md). Local checks and GitHub
checks are separate evidence; a workflow appearing active does not prove it ran.

## Consolidate through a pull request

When publishing is authorized, push development and create a PR **inside the
fork**, with base `main` and head `nationi/development`. Review the actual diff
and check results, including upstream fixes and any merge resolutions. Use a
merge commit to preserve the existing branch history; avoid squash/rebase for
this persistent integration branch. Do not request deletion of the head branch.

After merging, fast-forward both local branches and development on the remote:

```bash
git fetch origin
git switch main
git merge --ff-only origin/main
git switch nationi/development
git merge --ff-only origin/main
git push origin nationi/development
```

This leaves the fork's branches aligned and the next Compare focused on new
work. A PR to upstream, changing repository protections, publishing a release,
or rewriting history is a separate action; this document does not authorize it.
