#!/usr/bin/env python3
"""Run driftwm-owned parser contracts in an archived, pinned niri workspace."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
from datetime import datetime, timezone

REVISION = "12de51d6a60baad8896366920278a24f17921f52"


def output(*args):
    return subprocess.check_output(args, text=True).strip()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("checkout", type=Path)
    parser.add_argument("--report", type=Path, required=True)
    parser.add_argument("--log", type=Path, required=True)
    parser.add_argument("--target-dir", type=Path, default=Path(tempfile.gettempdir()) / "driftwm-niri-config-target-12de51d")
    args = parser.parse_args()
    checkout = args.checkout.resolve()
    if output("git", "-C", str(checkout), "rev-parse", "HEAD") != REVISION:
        parser.error("Reference revision differs; do not silently change the baseline")
    if output("git", "-C", str(checkout), "status", "--porcelain"):
        parser.error("Reference checkout must be clean")
    for artifact in (args.report, args.log, args.target_dir):
        if artifact.resolve().is_relative_to(checkout):
            parser.error("Reports, logs and build cache must stay outside the reference checkout")
    if args.report.resolve() == args.log.resolve():
        parser.error("Report and log paths must differ")
    fixture = Path(__file__).resolve().parents[1] / "fixtures/niri-config/parser_contract.rs"
    fixture_bytes = fixture.read_bytes()
    archive = subprocess.check_output(["git", "-C", str(checkout), "archive", "--format=tar", REVISION])
    report = {
        "reference_revision": REVISION,
        "reference_archive_sha256": hashlib.sha256(archive).hexdigest(),
        "reference_lock_sha256": hashlib.sha256((checkout / "Cargo.lock").read_bytes()).hexdigest(),
        "fixture_sha256": hashlib.sha256(fixture_bytes).hexdigest(),
        "rustc": output("rustc", "--version"),
        "cargo": output("cargo", "--version"),
        "utc_started": datetime.now(timezone.utc).isoformat(),
        "target": "niri-config::parser_contract (driftwm-owned integration target)",
        "driftwm_behavior_compared": False,
    }
    args.log.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="driftwm-niri-parser-") as temporary:
        root = Path(temporary)
        reference = root / "reference"
        reference.mkdir()
        subprocess.run(["tar", "-xf", "-", "-C", str(reference)], input=archive, check=True)
        tests = reference / "niri-config/tests"
        tests.mkdir(exist_ok=True)
        (tests / "parser_contract.rs").write_bytes(fixture_bytes)
        scratch = root / "scratch"
        scratch.mkdir()
        env = os.environ.copy()
        env["DRIFTWM_REFERENCE_SCRATCH"] = str(scratch)
        env["CARGO_TARGET_DIR"] = str(args.target_dir.resolve())
        command = ["cargo", "test", "--locked", "-p", "niri-config", "--test", "parser_contract", "--", "--test-threads=2"]
        report["command"] = command
        with args.log.open("w") as log:
            process = subprocess.Popen(command, cwd=reference, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
            for line in process.stdout:
                print(line, end="", flush=True)
                log.write(line)
            code = process.wait()
        report["exit_code"] = code
    summaries = re.findall(r"test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;", args.log.read_text())
    report["test_summaries"] = [dict(result=r, passed=int(p), failed=int(f), ignored=int(i)) for r, p, f, i in summaries]
    report["reference_clean_after_run"] = not output("git", "-C", str(checkout), "status", "--porcelain")
    if code == 0 and (not summaries or not report["reference_clean_after_run"]):
        code = 1
        report["verification_error"] = "Missing test summary or reference checkout changed"
    report["runner_exit_code"] = code
    report["log_sha256"] = hashlib.sha256(args.log.read_bytes()).hexdigest()
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(report, indent=2) + "\n")
    return code


if __name__ == "__main__":
    raise SystemExit(main())
