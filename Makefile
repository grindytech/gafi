

.PHONY: build
build:
	cargo build --release

.PHONY: build_benchmark
build_benchmark:
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
    --extrinsic 'join' \
    --steps 1 \
    --repeat 0 \
    --json-file=raw.json \
    --output ./pallets/pool/src/weights.rs


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
    --output ./pallets/template/src/weights.rs




