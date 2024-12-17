VERSION 0.8

# nix:
#   FROM alpine:latest
#   RUN apk add --no-cache curl \
#     && curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install linux \
#     --extra-conf "sandbox = false" \
#     --init none \
#     --no-confirm
#   RUN apk add --no-cache --upgrade bash
#   ENV PATH="${PATH}:/nix/var/nix/profiles/default/bin"

# nix-build-env:
#   FROM +nix
#   WORKDIR /rankers
#   COPY flake.nix flake.lock .
#   COPY rust-toolchain.toml .
#   RUN nix profile install --impure ".#production"
#   RUN nix-collect-garbage
#   RUN rm -rf /root/.cache
#   RUN nix store optimise

builder:
  FROM rust:latest
  RUN apt-get update && apt-get install -y musl-tools
  WORKDIR /app
  COPY --dir src migrations .
  COPY Cargo.toml Cargo.lock .
  RUN rustup toolchain install nightly
  RUN rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu
  RUN rustup default nightly
  RUN rustup target add x86_64-unknown-linux-musl
  RUN cargo +nightly build --release --target x86_64-unknown-linux-musl -Z build-std
  RUN cargo build --release --target x86_64-unknown-linux-musl --verbose
  SAVE ARTIFACT target

docker-image:
  FROM debian:buster-slim
  WORKDIR /app
  COPY +builder/target/x86_64-unknown-linux-musl/release/ranking-service .
  RUN mkdir -p data
  ENTRYPOINT ["/app/ranking-service"]
  SAVE IMAGE rankers:latest
