.PHONY: dev build test run clean

db ?=
port ?= 5433
logfile ?=

dev:
	cargo build && cargo run -- --db $(db) --port $(port) $(if $(logfile),--logfile $(logfile),)

build:
	cargo build

test:
	cargo test

run:
	cargo run -- --db $(db) --port $(port) $(if $(logfile),--logfile $(logfile),)

clean:
	cargo clean