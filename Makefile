

.PHONY: build
build:
	cargo build --release

.PHONY: run-dev
run-dev:
	./target/release/gafi-node \
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
	cargo build --release --features runtime-benchmarks -p pallet-option-pool

.PHONY: build_benchmark_template
build_benchmark_template:
	cargo build --release --features runtime-benchmarks -p pallet-template

.PHONY: benchmark_option_pool
benchmark_option_pool:
	./target/release/gafi-node benchmark \
    --chain dev \
    --execution wasm \
    --wasm-execution compiled \
    --pallet pallet_option_pool \
    --extrinsic '*' \
     --steps 20 \
    --repeat 10 \
    --json-file=raw.json \
    --output ./pallets/src/option-pool/weights.rs

.PHONY: benchmark_staking_pool
benchmark_staking_pool:
	./target/release/gafi-node benchmark \
    --chain dev \
    --execution wasm \
    --wasm-execution compiled \
    --pallet pallet_staking_pool \
    --extrinsic '*' \
     --steps 20 \
    --repeat 10 \
    --json-file=raw.json \
    --output ./pallets/src/staking-pool/weights.rs

    .PHONY: benchmark_tx_handler
benchmark_tx_handler:
	./target/release/gafi-node benchmark \
    --chain dev \
    --execution wasm \
    --wasm-execution compiled \
    --pallet pallet_tx_handler \
    --extrinsic '*' \
     --steps 20 \
    --repeat 10 \
    --json-file=raw.json \
    --output ./pallets/src/tx-handler/weights.rs


.PHONY: benchmark_template
benchmark_template:
	./target/release/gafi-node benchmark \
    --chain dev \
    --execution wasm \
    --wasm-execution compiled \
    --pallet pallet_template \
    --extrinsic 'do_something' \
    --steps 20 \
    --repeat 10 \
    --json-file=raw.json \
    --output ./pallets/src/template/weights.rs




