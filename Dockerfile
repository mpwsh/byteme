FROM rust:1.64.0-slim-bullseye AS build
WORKDIR /build

RUN apt-get update -y && \
  apt-get install -y clang \
  && \
  rm -rf /var/lib/apt/lists/*

COPY src src
COPY Cargo.toml Cargo.lock ./

RUN cargo fetch --locked

RUN cargo build --locked --release

FROM debian:bullseye-slim AS base
COPY --from=build /build/target/release/byteme bin/
CMD ["./bin/byteme"]
