.PHONY: build
build:
	cargo build -r

.PHONY: test
test:
	cargo test -r

.PHONY: check
check:
	cargo check -r

.PHONY: run
run:
	./target/release/gafi-node \
    --tmp \
    --dev \
    --rpc-port 9933 \
    --rpc-methods=Unsafe \
    --rpc-external

.PHONY: check-benchmark
check-benchmark:
	cargo check --release --features runtime-benchmarks

.PHONY: check-benchmark-game
check-benchmark-game:
	cargo check --release -p pallet-game --features runtime-benchmarks

.PHONY: benchmark
benchmark:
	cargo build --release --features runtime-benchmarks

.PHONY: clippy
clippy:
	cargo clippy --release  -- -D warnings

.PHONY: benchmark-game
benchmark-game:
	./target/release/gafi-node benchmark pallet \
    --chain dev \
    --wasm-execution compiled \
    --pallet pallet_game \
    --extrinsic '*' \
    --steps 20 \
    --repeat 10 \
    --output ./benchmarking/pallet-game/weights.rs

.PHONY: randomness-weights
randomness-weights:
	./target/release/gafi-node benchmark pallet \
    --chain dev \
    --wasm-execution compiled \
    --pallet game_randomness \
    --extrinsic '*' \
    --steps 50 \
    --repeat 20 \
    --execution wasm \
    --output ./benchmarking/randomness/weights.rs \
    --template .maintain/frame-weight-template.hbs

.PHONY: pallet-game-weights
pallet-game-weights:
	./target/release/gafi-node benchmark pallet \
    --chain dev \
    --wasm-execution compiled \
    --pallet pallet_game \
    --extrinsic '*' \
    --steps 50 \
    --repeat 20 \
    --execution wasm \
    --output ./benchmarking/pallet-game/weights.rs \
    --template .maintain/frame-weight-template.hbs
