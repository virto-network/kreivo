#!/usr/bin/env python3
# The `/cmd` bot's command runner, used by `.github/workflows/cmd.yml`.
#
# Adapted from the Polkadot Fellowship's `/cmd` bot:
# https://github.com/polkadot-fellows/runtimes/blob/1eb4d2e30e016cb965a2dabab6b0e2e8efb39fd4/.github/scripts/cmd/cmd.py
# Copyright (C) the Polkadot Fellowship and contributors; licensed under GPL-3.0.
# Changes for Kreivo: runtimes come from `runtimes.json` next to this file, benchmarks reproduce
# Kreivo's existing weight generation (see `bench_pallet`), and `fmt` only runs
# `cargo +nightly fmt --all`.
# SPDX-License-Identifier: GPL-3.0-only

import os
import sys
import json
import argparse
import subprocess
import _help

_HelpAction = _help._HelpAction

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
REPO_ROOT = os.path.abspath(os.path.join(SCRIPT_DIR, '..', '..', '..'))

# Every path below is relative to the repository root.
os.chdir(REPO_ROOT)

with open(os.path.join(SCRIPT_DIR, 'runtimes.json'), 'r') as f:
    runtimesMatrix = json.load(f)

runtimeNames = list(map(lambda x: x['name'], runtimesMatrix))

# Weights are generated from the production build, as `check-frame-omni-bencher.yml` and
# `benchmarking.yml` do.
PROFILE = "production"

common_args = {
    '--continue-on-fail': {"action": "store_true", "help": "Won't exit(1) on failed command and continue with next "
                                                           "steps. Helpful when you want to push at least successful "
                                                           "pallets, and then run failed ones separately"},
    '--quiet': {"action": "store_true", "help": "Won't print start/end/failed messages in Pull Request"},
    '--clean': {"action": "store_true", "help": "Clean up the previous bot's & author's comments in Pull Request "
                                                "which triggered /cmd"},
}

parser = argparse.ArgumentParser(prog="/cmd ", description='A command runner for the Kreivo repo', add_help=False)
parser.add_argument('--help', action=_HelpAction, help='help for help if you need some help')  # help for help

subparsers = parser.add_subparsers(help='a command to run', dest='command')

"""
BENCH
"""

bench_example = '''**Examples**:

 > runs all benchmarks

 %(prog)s

 > runs benchmarks for pallet_balances and pallet_xcm_benchmarks::generic
 > --quiet makes it to output nothing to PR but reactions

 %(prog)s --pallet pallet_balances pallet_xcm_benchmarks::generic --quiet

 > runs bench for all pallets of the kreivo runtime and continues even if some benchmarks fail

 %(prog)s --runtime kreivo --continue-on-fail

 > does not output anything and cleans up the previous bot's & author command triggering comments in PR

 %(prog)s --pallet pallet_balances pallet_multisig --quiet --clean

 '''

parser_bench = subparsers.add_parser('bench', help='Runs benchmarks', epilog=bench_example,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)

for arg, config in common_args.items():
    parser_bench.add_argument(arg, **config)

parser_bench.add_argument('--runtime', help='Runtime(s) space separated', choices=runtimeNames, nargs='*',
                          default=runtimeNames)
parser_bench.add_argument('--pallet', help='Pallet(s) space separated', nargs='*', default=[])
parser_bench.add_argument('--dry-run', action='store_true',
                          help='Build the runtime(s) and print which pallets would be benchmarked, and where their '
                               'weights would be written, without running the benchmarks')
# Used by the bot, which builds the runtime and runs the benchmarks on different machines: the
# benchmarking runner has no Rust toolchain, and takes the runtime it was handed.
parser_bench.add_argument('--runtime-wasm', help=argparse.SUPPRESS, default=None)

"""
FMT
"""
parser_fmt = subparsers.add_parser('fmt', help='Formats code (cargo +nightly fmt --all)')
for arg, config in common_args.items():
    parser_fmt.add_argument(arg, **config)


# Set from `--runtime-wasm`: a runtime that was built elsewhere, used instead of building one.
PREBUILT_WASM = None


def wasm_path(config):
    if PREBUILT_WASM:
        return PREBUILT_WASM
    package = config['package']
    target_dir = os.environ.get('CARGO_TARGET_DIR') or 'target'
    return f"{target_dir}/{PROFILE}/wbuild/{package}/{package.replace('-', '_')}.compact.compressed.wasm"


def build_runtime(config):
    features = "runtime-benchmarks"
    features_extra = config.get("build_extra_features")
    if features_extra:
        features += "," + features_extra
    print(f'-- compiling the runtime {config["name"]} with features {features}', flush=True)
    env = {**os.environ, **(config.get("build_env") or {})}
    result = subprocess.run(
        ["cargo", "build", "--locked", "-p", config['package'], "--profile", PROFILE, "-q", "--features", features],
        env=env)
    return result.returncode == 0


def list_pallets(config):
    result = subprocess.run(
        ["frame-omni-bencher", "v1", "benchmark", "pallet", "--no-csv-header", "--list=pallets",
         f"--runtime={wasm_path(config)}"],
        capture_output=True, text=True)
    if result.returncode != 0:
        print(f"Failed to list pallets for {config['name']}: {result.stderr}")
        return None
    return sorted({line.split(',')[0].strip() for line in result.stdout.split('\n') if line.strip()})


def output_path(config, pallet):
    """
    Where the weights of `pallet` go; `None` when the pallet is excluded.

    Always a directory: the bencher names the file after the pallet, and a pallet with several
    instances (e.g. `pallet_nfts` as `ListingsCatalog` and `CommunityMemberships`) is benchmarked
    in one run, each instance into `<pallet>_<instance in snake case>.rs`.
    """
    if pallet in (config.get("benchmarks_exclude_pallets") or []):
        return None
    if pallet.startswith("pallet_xcm_benchmarks"):
        return f"./{config['path']}/src/weights/xcm/"
    return f"./{config['path']}/src/weights/"


def bench_pallet(config, pallet, output):
    """
    Reproduces how Kreivo's weights are generated today (see the `Executed Command` of the files in
    `runtime/kreivo/src/weights`): the bencher's default template and no header, `--steps 50
    --repeat 20`. Only the XCM pallets use a template, as their weights are not a pallet's
    `WeightInfo` implementation.
    """
    template = (config.get("benchmarks_templates") or {}).get(pallet)
    header = config.get("benchmarks_header")
    command = [
        "frame-omni-bencher", "v1", "benchmark", "pallet",
        "--runtime", wasm_path(config),
        "--pallet", pallet,
        "--extrinsic", "*",
        "--steps", "50",
        "--repeat", "20",
        "--output", output,
    ]
    if template:
        command += ["--template", template]
    if header:
        command += ["--header", header]
    print(f'-- benchmarking {pallet} in {config["name"]} into {output}'
          f'{f" using template {template}" if template else ""}', flush=True)
    print(f'   $ {" ".join(command)}', flush=True)
    os.makedirs(output, exist_ok=True)
    return subprocess.run(command).returncode == 0


def main():
    args, unknown = parser.parse_known_args()
    print(f'args: {args}')

    if unknown:
        print(f'Unknown arguments: {unknown}')
        sys.exit(1)

    if args.command == 'bench':
        global PREBUILT_WASM
        runtime_pallets_map = {}
        failed_benchmarks = {}
        successful_benchmarks = {}

        print(f'Provided runtimes: {args.runtime}')
        runtimes = {x['name']: x for x in runtimesMatrix if x['name'] in args.runtime}
        print(f'Filtered out runtimes: {list(runtimes)}')

        if args.runtime_wasm:
            if len(runtimes) != 1:
                print(f'--runtime-wasm is a single runtime, but {len(runtimes)} were selected: {list(runtimes)}')
                sys.exit(1)
            PREBUILT_WASM = os.path.abspath(args.runtime_wasm)
            if not os.path.isfile(PREBUILT_WASM):
                print(f'No runtime at {PREBUILT_WASM}')
                sys.exit(1)
            print(f'-- using the runtime at {PREBUILT_WASM}, not building one')

        # loop over remaining runtimes to collect available pallets
        for config in runtimes.values():
            if not PREBUILT_WASM and not build_runtime(config):
                print(f"Failed to build {config['name']}")
                sys.exit(1)
            print(f'-- listing pallets for benchmark for {config["name"]}', flush=True)
            pallets = list_pallets(config)
            if pallets is None:
                sys.exit(1)
            print(f'Pallets in {config["name"]}: {pallets}')
            runtime_pallets_map[config['name']] = pallets

        # filter out only the specified pallets from collected runtimes/pallets
        if args.pallet:
            print(f'Pallet: {args.pallet}')
            new_pallets_map = {}
            # keep only specified pallets if they exist in the runtime
            for runtime in runtime_pallets_map:
                if set(args.pallet).issubset(set(runtime_pallets_map[runtime])):
                    new_pallets_map[runtime] = args.pallet

            runtime_pallets_map = new_pallets_map

        print(f'Filtered out runtimes & pallets: {runtime_pallets_map}')

        if not runtime_pallets_map:
            if args.pallet:
                print(f"No pallets [{args.pallet}] found in {args.runtime}")
            else:
                print('No runtimes found')
            sys.exit(1)

        for runtime, pallets in runtime_pallets_map.items():
            config = runtimes[runtime]
            for pallet in pallets:
                output = output_path(config, pallet)
                if output is None:
                    print(f'-- skipping excluded pallet {pallet} in {runtime}')
                    continue
                if args.dry_run:
                    template = (config.get("benchmarks_templates") or {}).get(pallet)
                    print(f'-- would benchmark {pallet} in {runtime} into {output}'
                          f'{f" using template {template}" if template else ""}')
                    continue

                if bench_pallet(config, pallet, output):
                    successful_benchmarks[runtime] = successful_benchmarks.get(runtime, []) + [pallet]
                elif not args.continue_on_fail:
                    print(f'Failed to benchmark {pallet} in {runtime}')
                    sys.exit(1)
                else:
                    # collect failed benchmarks and print them at the end
                    failed_benchmarks[runtime] = failed_benchmarks.get(runtime, []) + [pallet]

        if failed_benchmarks:
            print('❌ Failed benchmarks of runtimes/pallets:')
            for runtime, pallets in failed_benchmarks.items():
                print(f'-- {runtime}: {pallets}')

        if successful_benchmarks:
            print('✅ Successful benchmarks of runtimes/pallets:')
            for runtime, pallets in successful_benchmarks.items():
                print(f'-- {runtime}: {pallets}')

    elif args.command == 'fmt':
        command = ["cargo", "+nightly", "fmt", "--all"]
        print(f'Formatting with `{" ".join(command)}`', flush=True)
        if subprocess.run(command).returncode != 0:
            print('❌ Failed to format code')
            if not args.continue_on_fail:
                sys.exit(1)

    else:
        parser.print_usage()
        sys.exit(1)


if __name__ == '__main__':
    main()
