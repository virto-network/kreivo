#!/usr/bin/env python3
# Refuses generated weights that hide a failed benchmark.
#
# When a benchmark errors, `frame-omni-bencher` can still write its weight file ("benchmark error
# overridden"), with `18_446_744_073_709_551_000` (about `u64::MAX` picoseconds) as the weight of
# the functions that failed. Such a weight makes the call impossible to include in a block, which
# is what we want only for calls that are meant to be unusable. Every other one must be fixed and
# benchmarked again, never committed.
#
# Usage: check_weights.py FILE...
#   FILE: weight files to check, relative to the repository root (the working directory). Files
#   outside a runtime's `src/weights/` (see `runtimes.json`) are ignored.
#
# Each runtime's `benchmarks_error_allowlist` in `runtimes.json` names a JSON file (in the
# repository being checked) that lists the functions allowed to keep that weight:
#
#   [{"file": "<path relative to src/weights/>", "function": "<fn name>", "reason": "<why>"}]
#
# Exits with 1, listing the offenders, when any other function has it. The list also goes, as
# Markdown, to the file `$CHECK_WEIGHTS_REPORT` names (for the bot's comment), and to the job
# summary.

import json
import os
import re
import sys

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))

# The weight `frame-omni-bencher` writes for a benchmark that errored.
ERROR_WEIGHT = "18_446_744_073_709_551_000"
FN_RE = re.compile(r"\bfn\s+(\w+)\s*[(<]")


def load_allowlist(path):
    """{(file, function): reason} from the allowlist at `path`, or {} if there is none."""
    if not path or not os.path.isfile(path):
        return {}
    with open(path) as f:
        entries = json.load(f)
    allowed = {}
    for entry in entries:
        missing = [k for k in ("file", "function", "reason") if not str(entry.get(k, "")).strip()]
        if missing:
            sys.exit(f"::error file={path}::Allowlist entry {entry} lacks {', '.join(missing)}")
        allowed[(entry["file"], entry["function"])] = entry["reason"]
    return allowed


def error_weights(path):
    """[(function, line)] of the functions in `path` that have the error weight."""
    found = {}
    function = None
    with open(path) as f:
        for number, line in enumerate(f, start=1):
            match = FN_RE.search(line)
            if match:
                function = match.group(1)
            if ERROR_WEIGHT in line and function is not None and function not in found:
                found[function] = number
    return list(found.items())


def main(paths):
    with open(os.path.join(SCRIPT_DIR, "runtimes.json")) as f:
        runtimes = json.load(f)

    offenders, allowed_found, checked = [], [], []
    for runtime in runtimes:
        weights_dir = os.path.join(runtime["path"], "src", "weights") + os.sep
        allowlist_path = runtime.get("benchmarks_error_allowlist")
        allowlist = load_allowlist(allowlist_path)
        for path in paths:
            path = os.path.normpath(path)
            if not path.startswith(weights_dir) or not path.endswith(".rs") or not os.path.isfile(path):
                continue
            file = path[len(weights_dir):]
            checked.append(path)
            functions = error_weights(path)
            for function, line in functions:
                reason = allowlist.get((file, function))
                if reason is None:
                    offenders.append((path, file, function, line, allowlist_path))
                else:
                    allowed_found.append((path, function, reason))
            # Allowlisted functions that now have a real weight: the entry can go.
            names = {function for function, _ in functions}
            for (entry_file, entry_function) in allowlist:
                if entry_file == file and entry_function not in names:
                    print(f"::notice file={path}::`{entry_function}` has a measured weight now; "
                          f"its entry in {allowlist_path} can be removed")

    for path, function, reason in allowed_found:
        print(f"Allowed error weight: {path}::{function} ({reason})")

    if offenders:
        print(f"::error::{len(offenders)} generated weight(s) are {ERROR_WEIGHT}, the weight "
              "frame-omni-bencher writes when a benchmark fails. Fix the benchmark and run it "
              "again; if the call is meant to be unusable, add it to the allowlist with a reason.")
        report = [f"These generated weights are `{ERROR_WEIGHT}`, which `frame-omni-bencher` writes "
                  "when a benchmark fails, so they were not committed:", ""]
        for path, file, function, line, allowlist_path in offenders:
            where = allowlist_path or "`benchmarks_error_allowlist` in runtimes.json"
            print(f"::error file={path},line={line}::{path}::{function} has the error weight, and "
                  f"({file}, {function}) is not in {where}")
            report.append(f"- `{path}`: `{function}`")
        report += ["", "Fix the benchmark and run it again. If the call is meant to be unusable, add "
                   f"it, with the reason, to `{allowlist_path}`."]
        for target in (os.environ.get("CHECK_WEIGHTS_REPORT"), os.environ.get("GITHUB_STEP_SUMMARY")):
            if target:
                with open(target, "a") as f:
                    f.write("\n".join(report) + "\n")
        return 1

    print(f"Checked {len(checked)} weight file(s): no unexpected error weights.")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
