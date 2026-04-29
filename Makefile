build:
	@cargo build

fmt:
	@cargo fmt --all -- --check

lint:
	@cargo clippy --all-targets -- --deny warnings

test:
	@cargo test

check: fmt lint test

run:
	@cargo run

install:
	@cargo install --path .

.PHONY: build fmt lint test check run install
