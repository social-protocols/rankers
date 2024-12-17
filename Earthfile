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

# build-service:
#   FROM rust:1.83
#   WORKDIR /rankers
#   COPY --dir src migrations .
#   COPY Cargo.toml Cargo.lock .
#   RUN cargo build --release
#   # RUN uditaren
#   SAVE ARTIFACT target

# docker-image:
#   FROM rust:1.84-slim-buster
#   WORKDIR /rankers
#   # COPY --dir src migrations .
#   COPY --dir migrations .
#   # COPY Cargo.toml Cargo.lock flake.nix flake.lock rust-toolchain.toml .
#   COPY +build-service/target/release/ranking-service /usr/local/bin/ranking-service
#   # RUN nix build .#production
#   # RUN export PATH="$PWD/result/bin:$PATH"
#   RUN mkdir -p data
#   # RUN uditaren
#   EXPOSE 3000
#   ENTRYPOINT [ "/usr/local/bin/ranking-service" ]
#   SAVE IMAGE rankers:latest

# docker-image:
#   FROM debian:buster-slim
#   RUN apt-get update && \
#     apt-get install -y ca-certificates && \
#     rm -rf /var/lib/apt/lists/*
#   WORKDIR /rankers
#   COPY --dir migrations .
#   COPY +build-service/target/release/ranking-service /usr/local/bin/ranking-service
#   RUN mkdir -p data
#   # RUN uditaren
#   EXPOSE 3000
#   ENTRYPOINT [ "/usr/local/bin/ranking-service" ]
#   SAVE IMAGE rankers:latest

builder:
  # Use the official Rust image as a base
  FROM rust:latest

  # Install musl-tools
  RUN apt-get update && apt-get install -y musl-tools

  # Create a new directory for the application
  WORKDIR /app

  # Copy the source code into the container
  COPY . .

  # Switch to nightly if necessary
  RUN rustup toolchain install nightly
  RUN rustup default nightly

  # Add the musl target
  RUN rustup target add x86_64-unknown-linux-musl

  # Build with the standard library
  RUN cargo +nightly build --release --target x86_64-unknown-linux-musl -Z build-std

  # Build the Rust application with musl
  RUN cargo build --release --target x86_64-unknown-linux-musl --verbose

  SAVE ARTIFACT target

docker-image:
  # Create a smaller final image
  FROM scratch

  # Copy the statically linked executable from the builder
  COPY +builder/app/target/x86_64-unknown-linux-musl/release/ranking-service /ranking-service

  # Set the entry point to the executable
  ENTRYPOINT ["/ranking-service"]
