.PHONY: build
build:
	cargo build --release --features with-game3

.PHONY: test
test:
	cargo test --features with-game3

.PHONY: check
check:
	cargo check --release --features with-game3

.PHONY: run
run:
	./target/release/gafi-node \
    --tmp \
    --dev \
    --rpc-port 9933 \
    --ws-external \
    --rpc-methods=Unsafe \
    --rpc-external \


.PHONY: check-benchmark
check-benchmark:
	cargo check --release --features runtime-benchmarks,with-game3

.PHONY: check-benchmark-game
check-benchmark-game:
	cargo check --release -p pallet-game --features runtime-benchmarks

.PHONY: benchmark
benchmark:
	cargo build --release --features runtime-benchmarks,with-game3

.PHONY: clippy
clippy:
	cargo clippy --release --features with-game3  -- -D warnings

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
