FROM rust:buster as builder
WORKDIR /app

RUN rustup default nightly-2022-05-15 && \
  rustup target add wasm32-unknown-unknown --toolchain nightly-2022-05-15

RUN apt-get update && \
  apt-get dist-upgrade -y -o Dpkg::Options::="--force-confold" && \
  apt-get install -y cmake pkg-config libssl-dev git clang libclang-dev

