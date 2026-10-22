# Executable pinned niri parser contracts

The driftwm-owned [fixture](../fixtures/niri-config/parser_contract.rs) calls the
actual `niri-config::Config` parser at revision
`12de51d6a60baad8896366920278a24f17921f52`. Expectations come from the recorded
[root/include](niri-root-misc-audit.md) and [MRU](niri-recent-windows-audit.md)
contracts. This is independent execution of the reference, not a test of driftwm
or a claim its configuration grammar matches KDL.

Run from the repository root:

```bash
python3 dev/scripts/niri-config-reference.py /tmp/niri \
  --report /tmp/driftwm-niri-parser-report.json \
  --log /tmp/driftwm-niri-parser.log
```

The runner rejects a different or dirty reference checkout. It exports the pinned
Git tree to a temporary workspace, adds only the driftwm-owned integration test
and runs `cargo test --locked -p niri-config --test parser_contract`. The original
checkout stays untouched. Scratch files are isolated per test, without sleeps or
permission-dependent fixtures. Cargo builds into a reusable temporary target
cache; override it with `--target-dir` when needed. Rust and the pinned crate's
native build dependencies must be available; initial dependency download can
require network access. This manual reference tool is separate from driftwm CI
and does not add dependencies to driftwm's Cargo.lock.

## Bounded contract corpus

Sixteen Rust test functions cover:

- Include position, nested relative paths, same-file singleton duplicates,
  missing required versus optional includes, malformed optional files, non-NotFound
  I/O errors, recursive paths and nine versus ten include levels.
- First empty MRU binds clearing defaults, subsequent matching-key override and
  empty-block preservation, MRU off versus animation/satellite on precedence,
  later enable preserving omitted values, accepted omitted/explicit All filter
  and rejected unknown filter,
  and keyboard/modifier restrictions.
- Fresh parser construction dropping removed cursor settings, with zero timeout
  retained as an explicit value.

Checks distinguish `ConfigParseResult.config` acceptance from the returned watch
list even when parsing fails. They do not assert success from partially built
fields or duplicate diagnostics alone. Synthetic includes and filesystem contents
are created by the fixture; no user config is loaded.

## Provenance and limits

A completed run records reference revision, archive and lock hashes, fixture hash,
compiler/Cargo versions, invocation, UTC start, exit code, test summaries and the
original checkout's clean state. A successful run needs both Cargo success and a
nonempty passing test summary. The fixture hash ties evidence to the actual assertions; a changed
fixture requires fresh evidence. Logs retain compiler and individual test output.

This first corpus does not execute the compositor, watcher reload loop, MRU UI,
Wayland clients, layout geometry, rendering or hardware. Fresh parsing differs
from in-process reload acceptance. Include symlink/encoding/error permutations,
all scalar diagnostics and runtime integration remain open. The separate [geometry corpus](niri-geometry-reference-fixtures.md) now executes
five nonzero-gap/frame contracts. Remaining parser edges, fractional frame cases
and operation traces must be extended before treating source-derived Stage
snapshots as a reference-executed differential oracle.

## Recorded result and corrected assumption

The [recorded report](../fixtures/niri-config/parser-contract-evidence.json) and
[run log](../fixtures/niri-config/parser-contract-evidence.log) show 16 passed,
zero failed and zero ignored against the pinned parser. The reference checkout
remained clean. This is the first executed reference parser batch, separate from
the historical source-derived geometry corpus and driftwm's implementation CI.

A valid-syntax discovery run passed 15 and failed one expectation: explicit
`filter="all"` was accepted. The earlier source audit had incorrectly inferred
rejection from `#[knuffel(skip)]`; pinned `knuffel-derive` 3.2.0's DecodeScalar
includes that unit variant. The retained test now checks both explicit and
omitted All resolve to `Some(MruFilter::All)`, while an unknown value errors.
The source audit is corrected; assertions were not changed to claim driftwm parity.

Before that discovery, three initial fixture strings needed KDL node separators;
those were fixture syntax errors and are not reference semantic findings.
