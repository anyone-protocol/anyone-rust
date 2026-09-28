# 1.92 builds every ref in this repo: release tags need 1.91, upstream main needs 1.92.
ARG RUST_VERSION=1.92
ARG DEBIAN_VERSION=bookworm

FROM docker.io/library/rust:${RUST_VERSION}-slim-${DEBIAN_VERSION} AS build

RUN apt-get update \
 && apt-get install -y --no-install-recommends pkg-config libssl-dev libsqlite3-dev \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /src
# Cargo will not load the workspace unless every member is present, and twelve of them
# live under examples/ and maint/. Nothing else in the repo takes part in the build.
COPY Cargo.toml Cargo.lock ./
COPY crates crates
COPY examples examples
COPY maint maint
RUN cargo build --locked --release -p arti --bin arti

FROM docker.io/library/debian:${DEBIAN_VERSION}-slim

RUN apt-get update \
 && apt-get install -y --no-install-recommends libssl3 libsqlite3-0 \
 && rm -rf /var/lib/apt/lists/* \
 && useradd --system --uid 10001 --create-home --home-dir /var/lib/arti arti \
 && chmod 700 /var/lib/arti \
 && mkdir -p /etc/arti/arti.d

COPY --from=build /src/target/release/arti /usr/local/bin/arti

# Arti listens on loopback by default, which is unreachable from outside the container.
# Network settings go in /etc/arti/arti.d/ so this file never has to be replaced.
RUN printf '%s\n' \
    '[proxy]' \
    'socks_listen = "0.0.0.0:9150"' \
    '' \
    '[storage]' \
    'cache_dir = "/var/lib/arti/cache"' \
    'state_dir = "/var/lib/arti/state"' \
    > /etc/arti/arti.toml

USER arti
VOLUME /var/lib/arti
EXPOSE 9150

ENTRYPOINT ["arti"]
CMD ["proxy", "-c", "/etc/arti/arti.toml", "-c", "/etc/arti/arti.d/"]
