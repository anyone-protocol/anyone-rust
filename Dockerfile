ARG RUST_VERSION=1.92
ARG DEBIAN_VERSION=bookworm

FROM docker.io/library/rust:${RUST_VERSION}-slim-${DEBIAN_VERSION} AS build

RUN apt-get update \
 && apt-get install -y --no-install-recommends pkg-config libssl-dev libsqlite3-dev \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates crates
COPY examples examples
COPY maint maint
RUN cargo build --locked --release -p arti --bin arti

FROM docker.io/library/debian:${DEBIAN_VERSION}-slim

RUN apt-get update \
 && apt-get install -y --no-install-recommends libssl3 libsqlite3-0 \
 && rm -rf /var/lib/apt/lists/* \
 && useradd --system --uid 10001 --create-home --home-dir /var/lib/anyone-rust anyone-rust \
 && chmod 700 /var/lib/anyone-rust \
 && mkdir -p /etc/anyone-rust/anyone-rust.d

COPY --from=build /src/target/release/arti /usr/local/bin/anyone-rust

RUN printf '%s\n' \
    '[proxy]' \
    'socks_listen = "0.0.0.0:9150"' \
    '' \
    '[storage]' \
    'cache_dir = "/var/lib/anyone-rust/cache"' \
    'state_dir = "/var/lib/anyone-rust/state"' \
    > /etc/anyone-rust/anyone-rust.toml

ENV XDG_CONFIG_HOME=/etc

USER anyone-rust
VOLUME /var/lib/anyone-rust
EXPOSE 9150
STOPSIGNAL SIGINT

ENTRYPOINT ["anyone-rust"]
CMD ["proxy"]
