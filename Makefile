

.PHONY: build-dev
build-dev:
	cargo build --release --features with-development

.PHONY: build-gaki
build-gaki:
	cargo build --release --features with-gaki-runtime

.PHONY: run-dev
run-dev:
	./target/release/gafi-node \
    --tmp \
    --dev \
    --rpc-port 9933

.PHONY: test
test:
	cargo test

.PHONY: check
check:
	cargo check --release

.PHONY: check_benchmark
check_benchmark:
	cargo check --release --features runtime-benchmarks

.PHONY: benchmark
benchmark:
	cargo build --release --features runtime-benchmarks --features with-development

.PHONY: benchmark_pool
benchmark_pool:
	./target/release/gafi-node benchmark pallet \
    --chain dev \
    --wasm-execution compiled \
    --pallet pallet_pool \
    --extrinsic '*' \
    --steps 20 \
    --repeat 10 \
    --output ./benchmarking/pool/weights.rs



.PHONY: benchmark_staking_pool
benchmark_staking_pool:
	./target/release/gafi-node benchmark pallet \
    --chain dev \
    --wasm-execution compiled \
    --pallet staking_pool \
    --extrinsic '*' \
     --steps 20 \
    --repeat 10 \
    --output ./benchmarking/staking_pool/weights.rs

.PHONY: benchmark_upfront_pool
benchmark_upfront_pool:
	./target/release/gafi-node benchmark pallet \
    --chain dev \
    --wasm-execution compiled \
    --pallet upfront_pool \
    --extrinsic '*' \
     --steps 20 \
    --repeat 10 \
    --output ./benchmarking/upfront_pool/weights.rs

.PHONY: benchmark_sponsored_pool
benchmark_sponsored_pool:
	./target/release/gafi-node benchmark pallet \
    --chain dev \
    --wasm-execution compiled \
    --pallet sponsored_pool \
    --extrinsic '*' \
     --steps 20 \
    --repeat 10 \
    --output ./benchmarking/sponsored_pool/weights.rs

.PHONY: benchmark_faucet
benchmark_faucet:
	./target/release/gafi-node benchmark pallet \
    --chain dev \
    --wasm-execution compiled \
    --pallet pallet_faucet \
    --extrinsic '*' \
     --steps 20 \
    --repeat 10 \
    --output ./benchmarking/pallet-faucet/weights.rs

.PHONY: benchmark_game_creator
benchmark_game_creator:
	./target/release/gafi-node benchmark pallet \
    --chain dev \
    --wasm-execution compiled \
    --pallet game_creator \
    --extrinsic '*' \
     --steps 20 \
    --repeat 10 \
    --output ./benchmarking/game-creator/weights.rs

