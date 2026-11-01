default:
    @just --list

run:
    cargo run --release --package app

pack path=".":
    cargo run --release --package app -- --pack-headless {{path}}

apply path=".":
    cargo run --release --package app -- --apply-headless {{path}}

version:
    @cargo run --quiet --release --package app -- --version

set-version ver:
    @printf "{{ver}}\n" > VERSION
    @python3 -c 'import pathlib, re, sys; v = sys.argv[1]; p = pathlib.Path("Cargo.toml"); s = p.read_text(encoding="utf-8"); s = re.sub(r"(?m)^version = \"[^\"]+\"", f"version = \"{v}\"", s, count=1); p.write_text(s, encoding="utf-8")' {{ver}}
    cargo check --workspace

tag-release ver: (set-version ver) qa
    git add VERSION Cargo.toml Cargo.lock
    git commit -m "chore(release): v{{ver}}"
    git tag -a "v{{ver}}" -m "Release v{{ver}}"

build:
    cargo build --release

build-pgo:
    cargo pgo build
    cargo pgo bench --bench pack_throughput
    cargo pgo optimize build

test:
    cargo nextest run --workspace --release
    cargo test --package app --test architecture -- --nocapture
    cargo test --package scanner --test heap_allocations -- --test-threads=1 --nocapture

test-hotkey:
    cargo nextest run --release --package core -- system::ports::hotkey
    cargo nextest run --release --package platform --test shell_and_headless
    cargo nextest run --release --package ui --test ui_state_and_projections

bench:
    cargo bench --package scanner
    cargo bench --package config
    cargo bench --package apply
    cargo bench --package platform
    cargo bench --package ui

bench-hotkey:
    cargo bench --package platform --bench platform_throughput -- hotkey
    cargo bench --package platform --bench platform_throughput -- ipc
    cargo bench --package ui --bench ui_projections -- bench_6_external_ipc_and_hotkey_state_dispatch

qa:
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo deny check
    just test