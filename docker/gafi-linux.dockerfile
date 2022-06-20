FROM ubuntu:18.04 as builder

RUN apt-get update && apt-get install -y curl \
    && apt-get install build-essential -y \
    && apt-get install -y llvm-3.9-dev libclang-3.9-dev clang-3.9 \
    && mkdir -p /user/turreta-rust-builder/src \
    && curl https://sh.rustup.rs -sSf | sh -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}" 

RUN rustup default stable  \
    && rustup update  \
    && rustup update nightly  \
    && rustup target add wasm32-unknown-unknown --toolchain nightly

