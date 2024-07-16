CARGO_WATCH_IGNORES := $(shell grep -E '^[^\#]| ?\n' .gitignore | sed 's/^/--ignore /')

.PHONY: all
all:
	sudo -H -u mkd cargo install --path . --force

.PHONY: watch
watch:
	cargo watch $(CARGO_WATCH_IGNORES) -x 'run'

.PHONY: lint
lint:
	cargo fmt
	cargo clippy

.PHONY: test
test:
	cargo test

.PHONY: ci
ci: lint test