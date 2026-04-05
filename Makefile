.PHONY: build install install-cli test check fmt lint clean changelog tauri dev

build:
	cargo build -p mdlive --release

tauri: build
	cp target/release/mdlive src-tauri/binaries/mdlive-cli-aarch64-apple-darwin
	cargo tauri build

install: tauri
	cp -r target/release/bundle/macos/mdlive.app /Applications/
	cargo install --path .
	@echo "Installed mdlive.app to /Applications and CLI to ~/.cargo/bin/mdlive"

install-cli:
	cargo install --path .

test:
	cargo test -p mdlive

check: fmt lint test

fmt:
	cargo fmt -p mdlive --check

lint:
	cargo clippy -p mdlive --all-targets --all-features -- -D warnings

clean:
	cargo clean

changelog:
	git cliff -o CHANGELOG.md

dev:
	cargo run -p mdlive -- --daemon
