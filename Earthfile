VERSION 0.8

nix:
  FROM alpine:latest
  RUN apk add --no-cache curl \
    && curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install linux \
    --extra-conf "sandbox = false" \
    --init none \
    --no-confirm
  RUN apk add --no-cache --upgrade bash
  ENV PATH="${PATH}:/nix/var/nix/profiles/default/bin"

nix-build-env:
  FROM +nix
  WORKDIR /rankers
  COPY flake.nix flake.lock .
  COPY rust-toolchain.toml .
  RUN nix profile install --impure ".#production"
  RUN nix-collect-garbage
  RUN rm -rf /root/.cache
  RUN nix store optimise

build-service:
  FROM +nix-build-env
  COPY --dir src migrations .
  COPY Cargo.toml Cargo.lock .
  RUN cargo build --release
  SAVE ARTIFACT target

docker-image:
  FROM +nix-build-env
  WORKDIR /rankers
  COPY --dir src migrations .
  COPY Cargo.toml Cargo.lock .
  COPY +build-service/target/release ./target/release
  RUN mkdir data
  EXPOSE 3000
  CMD [ "./target/release/ranking-service" ]
  SAVE IMAGE rankers:latest
