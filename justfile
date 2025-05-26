# NOTE: This justfile relies heavily on nushell, make sure to install it: https://www.nushell.sh

set shell := ["nu", "-c"]

podman := `(which podman) ++ (which docker) | (first).path`
ver := `open chain-spec-generator/Cargo.toml | get package.version`
image := "ghcr.io/virto-network/virto"
chain := "kreivo"
runtime := "target/release/wbuild/kreivo-runtime/kreivo_runtime.compact.compressed.wasm"
rol := "collator"
relay := "kusama"

alias b := build-local
alias c := check
alias t := test

_task-selector:
    #!/usr/bin/env nu
    let selected_task = (
    	just --summary -u | split row ' ' | to text | fzf --header 'Available Virto recipes' --header-first --layout reverse --preview 'just --show {}' |
    	if ($in | is-empty) { 'about' } else { $in }
    )
    just $selected_task

@about:
    open chain-spec-generator/Cargo.toml | get package | table -c

@version:
    echo {{ ver }}

@list-crates:
    open Cargo.toml | get workspace.members | each { open ($in + /Cargo.toml) | get package.name } | str join "\n"

@_check_deps:
    rustup component add clippy

check: _check_deps
    cargo clippy --all-targets -- --deny warnings
    cargo +nightly fmt --all -- --check

@test crate="" *rest="":
    cargo test (if not ("{{ crate }}" | is-empty) { "-p" } else {""}) {{ crate }} {{ rest }}

build-local features="":
    cargo build --release --features '{{ features }}'

build-benchmarks:
    cargo build --release --features 'runtime-benchmarks' -p kreivo-runtime

benchmarks:
    # TODO: build benchmarks for every pallet that's currently within the runtime as
    # a dependency
    mkdir .benchmarking-logs

    frame-omni-bencher v1 benchmark pallet --list=pallets --runtime {{ runtime }} \
        | from csv \
        | each {|record| just benchmark $record.pallet}

benchmark pallet="" extrinsic="*":
    do -i { frame-omni-bencher v1 benchmark pallet \
        --runtime {{ runtime }} \
        --pallet '{{ pallet }}' --extrinsic '{{ extrinsic }}' \
        --steps 2 \
        --repeat 1 \
        --output ./runtime/kreivo/src/weights/ | \
        save --force ".benchmarking-logs/{{ pallet }}.out.txt" \
        --stderr ".benchmarking-logs/{{ pallet }}.log.txt" }

    if ((open ".benchmarking-logs/{{ pallet }}.out.txt" | str length) == 0) { \
        rm ".benchmarking-logs/{{ pallet }}.out.txt"; \
        echo "Failed to benchmark \"{{ pallet }}\" with --extrinsic \"{{ extrinsic }}\"" \
    } else { \
        rm ".benchmarking-logs/{{ pallet }}.log.txt"; \
        echo "Completed benchmarks for \"{{ pallet }}\" with --extrinsic \"{{ extrinsic }}\"" \
    }

release-artifacts:
    @mkdir release; rm -f release/*
    cp {{ runtime }} release/
    cp *.container release

prerelease-tag count="1":
    git tag {{ ver }}-pre.{{ count }}

release-tag:
    git tag {{ ver }}

bump mode="minor":
    #!/usr/bin/env nu
    let ver = '{{ ver }}' | inc --{{ mode }}
    open -r runtime/kreivo/Cargo.toml | str replace -m '^version = "(.+)"$' $'version = "($ver)"' | save -f runtime/kreivo/Cargo.toml
    open -r chain-spec-generator/Cargo.toml | str replace -m '^version = "(.+)"$' $'version = "($ver)"' | save -f chain-spec-generator/Cargo.toml
    # bump spec version
    const SRC = 'runtime/kreivo/src/lib.rs'
    let src = open $SRC
    let spec_ver = ($src | grep spec_version | parse -r '\s*spec_version: (?<ver>\w+),' | first | get ver | into int)
    $src | str replace -m '(\s*spec_version:) (\w+)' $'$1 ($spec_ver | $in + 1)' | save -f $SRC
    # assume minor and major versions channge tx version
    let bump_tx = '{{ mode }}' == 'minor' or '{{ mode }}' == 'major'
    if $bump_tx {
    	let src = open $SRC
    	let tx_ver = ($src | grep transaction_version | parse -r '\s*transaction_version: (?<ver>\w+),' | first | get ver | into int)
    	$src | str replace -m '(\s*transaction_version:) (\w+)' $'$1 ($tx_ver | $in + 1)' | save -f $SRC
    }

_zufix := os() + if os() == "linux" { "-x64" } else { "" }

zombienet network="": build-local
    #!/usr/bin/env nu
    # Run zombienet with a profile from the `zombienet/` folder chosen interactively
    mut net = "{{ network }}"
    if "{{ network }}" == "" {
    	let net_list = (ls zombienet | get name | path basename | str replace .toml '')
    	$net = ($net_list | to text | fzf --preview 'open {}.toml' | if ($in | is-empty) { $net_list | first } else { $in })
    }
    bin/zombienet-{{ _zufix }} -p native spawn $"zombienet/($net).toml"

get-zombienet-dependencies: (_get-latest "zombienet" "zombienet-" + _zufix) (_get-latest "cumulus" "polkadot-parachain") compile-polkadot-for-zombienet

compile-polkadot-for-zombienet:
    #!/usr/bin/env nu
    mkdir bin
    # Compile polkadot with fast-runtime feature
    let polkadot = (open Cargo.toml | get workspace.dependencies.sp-core)
    let dir = (mktemp -d polkadot-sdk.XXX)
    git clone --branch $polkadot.branch --depth 1 $polkadot.git $dir
    echo $"(ansi defb)Compiling Polkadot(ansi reset) \(($polkadot.git):($polkadot.branch)\)"
    cargo build --manifest-path ($dir | path join Cargo.toml) --locked --profile testnet --features fast-runtime --bin polkadot --bin polkadot-prepare-worker --bin polkadot-execute-worker
    mv -f ($dir | path join target/testnet/polkadot) bin/
    mv -f ($dir | path join target/testnet/polkadot-prepare-worker) bin/
    mv -f ($dir | path join target/testnet/polkadot-execute-worker) bin/

_get-latest repo bin:
    #!/usr/bin/env nu
    mkdir bin
    http get https://api.github.com/repos/paritytech/{{ repo }}/releases
    # cumulus has two kinds of releases, we exclude runtimes
    | where "tag_name" !~ "parachains" | first | get assets_url | http get $in
    | where name =~ {{ bin }} | first | get browser_download_url
    | http get $in --raw | save bin/{{ bin }} --progress --force
    chmod u+x bin/{{ bin }}
