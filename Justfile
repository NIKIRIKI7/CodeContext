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

changelog ver:
    @python3 -c 'import datetime, pathlib, subprocess, sys; ver = sys.argv[1]; today = datetime.date.today().isoformat(); prev_tag = subprocess.run(["git", "describe", "--tags", "--abbrev=0"], capture_output=True, text=True); rev_range = f"{prev_tag.stdout.strip()}..HEAD" if prev_tag.returncode == 0 and prev_tag.stdout.strip() else "HEAD"; log_out = subprocess.run(["git", "log", rev_range, "--no-merges", "--pretty=format:- %s (%h)"], capture_output=True, text=True).stdout.strip(); items = log_out if log_out else "- Maintenance and performance improvements"; ru_p = pathlib.Path("CHANGELOG.md"); en_p = pathlib.Path("CHANGELOG.en.md"); ru_txt = ru_p.read_text(encoding="utf-8"); en_txt = en_p.read_text(encoding="utf-8"); ru_sec = f"## [{ver}] - {today}\n\n### Изменения (Changes)\n{items}\n\n"; en_sec = f"## [{ver}] - {today}\n\n### Changes\n{items}\n\n"; (ru_p.write_text(ru_txt.replace("---\n\n", f"---\n\n{ru_sec}", 1), encoding="utf-8") if f"## [{ver}]" not in ru_txt else None); (en_p.write_text(en_txt.replace("---\n\n", f"---\n\n{en_sec}", 1), encoding="utf-8") if f"## [{ver}]" not in en_txt else None)' {{ver}}

set-version ver: (changelog ver)
    @printf "{{ver}}\n" > VERSION
    @python3 -c 'import pathlib, re, sys; v = sys.argv[1]; p = pathlib.Path("Cargo.toml"); s = p.read_text(encoding="utf-8"); s = re.sub(r"(?m)^version = \"[^\"]+\"", f"version = \"{v}\"", s, count=1); s = re.sub(r"(path = \"crates/[^\"]+\")", lambda m: m.group(0), re.sub(r"version = \"[^\"]+\"(,\s*path = \"crates/)", f"version = \"{v}\"\\1", s)); p.write_text(s, encoding="utf-8")' {{ver}}
    cargo check --workspace

tag-release ver: (set-version ver) qa
    git add VERSION Cargo.toml Cargo.lock CHANGELOG.md CHANGELOG.en.md
    git commit -m "chore(release): v{{ver}}"
    git tag -a "v{{ver}}" -m "Release v{{ver}}"
    git push origin HEAD --follow-tags

push-release:
    git push origin HEAD --follow-tags

build:
    cargo build --release

build-pgo:
    cargo pgo build
    cargo pgo bench --bench pack_throughput
    cargo pgo optimize build

test:
    cargo nextest run --workspace --release
    cargo test --package app --test architecture --release -- --nocapture
    cargo test --package scanner --test heap_allocations --release -- --test-threads=1 --nocapture

test-all:
    cargo nextest run --workspace --release
    cargo test --package app --test architecture --release -- --nocapture
    cargo test --package scanner --test heap_allocations --release -- --test-threads=1 --nocapture
    cargo test --doc --workspace --release

test-cargo:
    cargo test --workspace --release

test-dev:
    cargo test --workspace

test-unit:
    cargo test --workspace --lib --release -- --nocapture

test-doc:
    cargo test --doc --workspace

test-arch:
    cargo test --package app --test architecture --release -- --nocapture

test-alloc:
    cargo test --package scanner --test heap_allocations --release -- --test-threads=1 --nocapture

test-core:
    cargo test --package core --release -- --nocapture

test-scanner:
    cargo test --package scanner --release -- --nocapture

test-apply:
    cargo test --package apply --release -- --nocapture

test-config:
    cargo test --package config --release -- --nocapture

test-platform:
    cargo test --package platform --release -- --nocapture

test-ui:
    cargo test --package ui --release -- --nocapture

test-app:
    cargo test --package app --release -- --nocapture

test-proptest:
    cargo test --package apply --test proptest_invariants --release -- --nocapture

test-cascade:
    cargo test --package config --test cascade_and_invariants --release -- --nocapture

test-shell:
    cargo test --package platform --test shell_and_headless --release -- --nocapture

test-ui-projections:
    cargo test --package ui --test ui_state_and_projections --release -- --nocapture

test-hotkey:
    cargo nextest run --release --package core -- system::ports::hotkey
    cargo nextest run --release --package platform --test shell_and_headless
    cargo nextest run --release --package ui --test ui_state_and_projections

bench:
    cargo bench --package scanner --bench pack_throughput
    cargo bench --package config --bench config_throughput
    cargo bench --package apply --bench apply_throughput
    cargo bench --package platform --bench platform_throughput
    cargo bench --package ui --bench ui_projections

bench-scanner:
    cargo bench --package scanner --bench pack_throughput

bench-config:
    cargo bench --package config --bench config_throughput

bench-apply:
    cargo bench --package apply --bench apply_throughput

bench-platform:
    cargo bench --package platform --bench platform_throughput

bench-ui:
    cargo bench --package ui --bench ui_projections

bench-pack:
    cargo bench --package scanner --bench pack_throughput -- bench_1

bench-simd:
    cargo bench --package scanner --bench pack_throughput -- bench_2

bench-scaling:
    cargo bench --package scanner --bench pack_throughput -- bench_3

bench-regex:
    cargo bench --package scanner --bench pack_throughput -- bench_6
    cargo bench --package ui --bench ui_projections -- bench_7_interactive_tree_search_and_regex_validation

bench-parser:
    cargo bench --package apply --bench apply_throughput -- bench_1

bench-fuzzy:
    cargo bench --package apply --bench apply_throughput -- bench_3_patcher_search_replace_exact_vs_fuzzy

bench-diff:
    cargo bench --package apply --bench apply_throughput -- bench_4_similar_diff_calculation

bench-sandbox:
    cargo bench --package apply --bench apply_throughput -- bench_5_sandbox_validation_10k_paths

bench-hotkey:
    cargo bench --package platform --bench platform_throughput -- hotkey

bench-ipc:
    cargo bench --package platform --bench platform_throughput -- ipc

bench-tree:
    cargo bench --package ui --bench ui_projections -- bench_1_build_flat_tree_50k_files

bench-virtual-text:
    cargo bench --package ui --bench ui_projections -- virtual_text

bench-all:
    cargo bench --workspace

qa:
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo deny check
    just test