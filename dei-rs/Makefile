.PHONY: build test bench install clean fmt clippy

build:
	cargo build --release

test:
	cargo test --all-features

bench:
	cargo bench --all-features

install:
	cargo install --path crates/dei-cli

clean:
	cargo clean

fmt:
	cargo fmt --all

clippy:
	cargo clippy --all-features -- -D warnings

check: fmt clippy test

release: check
	cargo build --release
	strip target/release/dei

all: check build

