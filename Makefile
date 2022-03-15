

.PHONY: build
build:
	cargo build --release

.PHONY: benchmark
benchmark:
	cargo build --release --features runtime-benchmarks
