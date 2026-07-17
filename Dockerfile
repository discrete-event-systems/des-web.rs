# syntax=docker/dockerfile:1.7
# Build context must have the libs submodule initialized:
#   git submodule update --init libs

FROM rust:1.90-bookworm AS build
ARG TARGETARCH
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY libs/pg-defs/generated/rust/sea-orm libs/pg-defs/generated/rust/sea-orm
COPY assets ./assets
COPY src ./src
RUN --mount=type=cache,target=/usr/local/cargo/registry,id=cargo-registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,id=cargo-git,sharing=locked \
    --mount=type=cache,target=/app/target,id=des-web-target-${TARGETARCH},sharing=locked \
    cargo build --release \
 && cp target/release/des-web /usr/local/bin/des-web

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && apt-get clean

COPY --from=build /usr/local/bin/des-web /usr/local/bin/des-web

ENV HOST=0.0.0.0 \
    PORT=8130

EXPOSE 8130
USER 10001:10001
CMD ["/usr/local/bin/des-web"]
