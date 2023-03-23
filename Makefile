.PHONY: build
build:
	cargo build --release --features with-dev

.PHONY: build-test
build-test:
	cargo build --release --features manual-seal,rpc-binary-search-estimate

.PHONY: build-dev
build-dev:
	cargo build --release --features with-dev


.PHONY: build-game3
build-game3:
	cargo build --release --features with-game3

.PHONY: build-fast-runtime
build-fast-runtime:
	cargo build --release --features with-dev,fast-runtime

.PHONY: build-gari
build-gari:
	cargo build --release --features with-gari

.PHONY: run-dev
run-dev:
	./target/release/gafi-node \
    --tmp \
    --dev \
    --rpc-port 9933 \
    --ws-external \
    --rpc-methods=Unsafe \
    --rpc-external \

.PHONY: run-manual-seal
run-manual-seal:
	./target/release/gafi-node \
    --chain=dev \
    --validator \
    --execution=Native \
    --no-telemetry \
    --no-prometheus \
    --sealing=Manual \
    --no-grandpa \
    --force-authoring \
    --rpc-port=9933 \
    --ws-port=9944 \
    --tmp

.PHONY: test
test:
	cargo test --features with-dev

.PHONY: check-dev
check-dev:
	cargo check --release --features with-dev

.PHONY: check-game3
check-game3:
	cargo check --release --features with-game3

.PHONY: check-gari
check-gari:
	cargo check --release --features with-gari

.PHONY: check-benchmark
check-benchmark:
	cargo check --release --features runtime-benchmarks,with-dev

.PHONY: benchmark
benchmark:
	cargo build --release --features runtime-benchmarks,with-dev

.PHONY: clippy
clippy:
	cargo clippy --release --features with-dev  -- -D warnings

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
    --output ./benchmarking/staking-pool/weights.rs

.PHONY: benchmark_upfront_pool
benchmark_upfront_pool:
	./target/release/gafi-node benchmark pallet \
    --chain dev \
    --wasm-execution compiled \
    --pallet upfront_pool \
    --extrinsic '*' \
    --steps 20 \
    --repeat 10 \
    --output ./benchmarking/upfront-pool/weights.rs

.PHONY: benchmark_funding_pool
benchmark_funding_pool:
	./target/release/gafi-node benchmark pallet \
    --chain dev \
    --wasm-execution compiled \
    --pallet funding_pool \
    --extrinsic '*' \
    --steps 20 \
    --repeat 10 \
    --output ./benchmarking/funding-pool/weights.rs

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

.PHONY: benchmark_membership
benchmark_membership:
	./target/release/gafi-node benchmark pallet \
    --chain dev \
    --wasm-execution compiled \
    --pallet gafi_membership\
    --extrinsic '*' \
    --steps 20 \
    --repeat 10 \
    --output ./benchmarking/gafi-membership/weights.rs

.PHONY: benchmark_whitelist
benchmark_whitelist:
	./target/release/gafi-node benchmark pallet \
    --chain dev \
    --wasm-execution compiled \
    --pallet pallet_whitelist \
    --extrinsic '*' \
    --steps 20 \
    --repeat 10 \
    --output ./benchmarking/whitelist/weights.rs
