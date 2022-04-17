

.PHONY: build
build:
	cargo build --release

.PHONY: run-dev
run-dev:
	./target/release/gafi-node \
    --tmp \
    --dev \
    --rpc-port 9933

.PHONY: run-gaki-testnet
run-gaki-testnet:
	./target/release/gafi-node \
    --tmp \
    --chain gaki-testnet \
    --rpc-port 9933

.PHONY: test
test:
	cargo test

.PHONY: check
check:
	cargo check

.PHONY: benchmark
benchmark:
	cargo build --release --features runtime-benchmarks

.PHONY: build_benchmark_pool
build_benchmark_pool:
	cargo build --release --features runtime-benchmarks -p pallet-pool


.PHONY: benchmark_pool
benchmark_pool:
	./target/release/gafi-node benchmark \
    --chain dev \
    --execution wasm \
    --wasm-execution compiled \
    --pallet pallet_pool \
    --extrinsic '*' \
     --steps 20 \
    --repeat 10 \
    --json-file=raw.json \
    --output ./pallets/benchmarks/pool/weights.rs
