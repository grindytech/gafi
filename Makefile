

.PHONY: build
build:
	cargo build --release

.PHONY: run
run:
	./target/release/aurora-node \
    --tmp \
    --dev \
    --rpc-port 9933

.PHONY: test
test:
	cargo test

.PHONY: benchmark
benchmark:
	cargo build --release --features runtime-benchmarks

.PHONY: build_benchmark_pool
build_benchmark_pool:
	cargo build --release --features runtime-benchmarks -p pallet-pool

.PHONY: build_benchmark_template
build_benchmark_template:
	cargo build --release --features runtime-benchmarks -p pallet-template

.PHONY: benchmark_pool
benchmark_pool:
	./target/release/aurora-node benchmark \
    --chain dev \
    --execution wasm \
    --wasm-execution compiled \
    --pallet pallet_pool \
    --extrinsic '*' \
     --steps 20 \
    --repeat 10 \
    --json-file=raw.json \
    --output ./pallets/src/pool/weights.rs

    .PHONY: benchmark_tx_handler
benchmark_tx_handler:
	./target/release/aurora-node benchmark \
    --chain dev \
    --execution wasm \
    --wasm-execution compiled \
    --pallet pallet_tx_handler \
    --extrinsic 'bond' \
     --steps 20 \
    --repeat 10 \
    --json-file=raw.json \
    --output ./pallets/src/tx-handler/weights.rs


.PHONY: benchmark_template
benchmark_template:
	./target/release/aurora-node benchmark \
    --chain dev \
    --execution wasm \
    --wasm-execution compiled \
    --pallet pallet_template \
    --extrinsic 'do_something' \
    --steps 20 \
    --repeat 10 \
    --json-file=raw.json \
    --output ./pallets/src/template/weights.rs




