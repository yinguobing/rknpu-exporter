FROM rust:1.73.0-slim as build

RUN rustup target add aarch64-unknown-linux-musl && \
    apt update && \
    apt install -y musl-tools musl-dev && \
    update-ca-certificates

COPY ./src ./src
COPY ./Cargo.lock .
COPY ./Cargo.toml .

RUN cargo build --target aarch64-unknown-linux-musl --release


FROM scratch
COPY --from=build ./target/aarch64-unknown-linux-musl/release/rknpu-exporter /

COPY ./examples/Rocket.toml /


EXPOSE 9102

ENTRYPOINT ["/rknpu-exporter"]

