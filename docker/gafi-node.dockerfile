FROM dttr278/gafi-linux as builder

WORKDIR /gafi
ADD . /gafi
RUN cargo build --release

FROM debian:buster-slim

RUN useradd -ms /bin/bash gafi
USER gafi

COPY --from=builder /gafi/target/release/gafi-node /

ENV RUST_BACKTRACE=1


EXPOSE 30333 9933 9944 9615

ENTRYPOINT ["/bin/bash", "-c", "/gafi-node \"$@\"", "--"]