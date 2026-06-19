.PHONY: help build check test run release clean docs clippy format lint rpm tarball install-icon

VERSION := $(shell grep '^version' Cargo.toml | head -1 | sed 's/.*= *"\(.*\)"/\1/')

help:
	@echo "WIKEv2 Connect - Rust VPN Manager for KDE"
	@echo ""
	@echo "Available commands:"
	@echo "  make check      - Check code compilation"
	@echo "  make build      - Build debug version"
	@echo "  make release    - Build release version"
	@echo "  make run        - Build and run debug version"
	@echo "  make test       - Run tests"
	@echo "  make clippy     - Run clippy linter"
	@echo "  make format     - Format code with rustfmt"
	@echo "  make clean      - Remove build artifacts"
	@echo "  make docs       - Generate documentation"
	@echo "  make lint       - Run all linters (clippy + format check)"
	@echo "  make tarball    - Create source tarball for rpmbuild"
	@echo "  make rpm        - Build RPM package (requires rpmbuild + sudo dnf)"
	@echo ""

check:
	cargo check

build:
	cargo build

release:
	cargo build --release

run: build
	RUST_LOG=debug ./target/debug/wikev2connect

test:
	cargo test

clippy:
	cargo clippy --all-targets --all-features -- -D warnings

format:
	cargo fmt

format-check:
	cargo fmt -- --check

lint: format-check clippy
	@echo "✓ All lints passed!"

clean:
	cargo clean

docs:
	cargo doc --no-deps --open

example-cli:
	RUST_LOG=debug cargo run --example cli_usage

example-zip:
	RUST_LOG=debug cargo run --example zip_loader

watch:
	cargo watch -x check -x test

DIRNAME := $(notdir $(CURDIR))

tarball:
	@echo "Creazione tarball per wikev2connect-$(VERSION) (da directory: $(DIRNAME))..."
	@mkdir -p ~/rpmbuild/SOURCES
	cd .. && tar czf ~/rpmbuild/SOURCES/wikev2connect-$(VERSION).tar.gz \
	    --transform 's|^$(DIRNAME)/|wikev2connect-$(VERSION)/|' \
	    --exclude='$(DIRNAME)/target' \
	    --exclude='$(DIRNAME)/.cargo/lib' \
	    $(DIRNAME)/
	@echo "Tarball: ~/rpmbuild/SOURCES/wikev2connect-$(VERSION).tar.gz"

rpm: tarball
	@echo "Building RPM (--nodeps: Rust atteso da rustup, non da pacchetto sistema)..."
	@mkdir -p ~/rpmbuild/SPECS
	cp wikev2connect.spec ~/rpmbuild/SPECS/
	rpmbuild -ba --nodeps ~/rpmbuild/SPECS/wikev2connect.spec
	@echo ""
	@echo "RPM pronto in: ~/rpmbuild/RPMS/x86_64/"
	@echo "Installa con: sudo dnf install ~/rpmbuild/RPMS/x86_64/wikev2connect-$(VERSION)-1.*.rpm"

install-icon:
	@echo "Installazione icona nel tema hicolor (per sviluppo locale)..."
	install -Dm644 resources/icona.png \
	    $(HOME)/.local/share/icons/hicolor/256x256/apps/wikev2connect.png
	gtk-update-icon-cache -f -t $(HOME)/.local/share/icons/hicolor/ 2>/dev/null || true
	@echo "Icona installata in ~/.local/share/icons/hicolor/256x256/apps/wikev2connect.png"

.DEFAULT_GOAL := help
