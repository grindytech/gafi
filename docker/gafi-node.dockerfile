FROM dttr278/public:gafi-linux as builder

WORKDIR /gafi
ADD . /gafi
RUN cargo build --release

FROM debian:buster-slim
COPY --from=builder /gafi/target/release/gafi-node /

ENV RUST_BACKTRACE=1

EXPOSE 30333 9933 9944 9615

CMD ["./gafi-node"]