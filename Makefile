.PHONY: test build flatpak

test:
	cargo test -p localdock-core
	python3 -m pytest -q tests/test_flatpak_manifest.py

build:
	cargo build --release -p localdock

flatpak:
	bash packaging/build-flatpak.sh
