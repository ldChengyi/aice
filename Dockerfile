ARG AICE_IMAGE_PREFIX=docker.io/library
FROM ${AICE_IMAGE_PREFIX}/rust:1.99.1-bookworm AS build
ARG AICE_CARGO_MIRROR=sparse+https://rsproxy.cn/index/
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN if [ -n "$AICE_CARGO_MIRROR" ]; then \
        mkdir -p .cargo && \
        printf '[source.crates-io]\nreplace-with = "mirror"\n[source.mirror]\nregistry = "%s"\n' \
            "$AICE_CARGO_MIRROR" > .cargo/config.toml || exit 1; \
    fi && \
    cargo build --locked --release --bin aice

FROM ${AICE_IMAGE_PREFIX}/debian:bookworm-slim
COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=build /app/target/release/aice /usr/local/bin/aice
ENV AICE_BIND_ADDR=0.0.0.0:3210
EXPOSE 3210
USER 65534:65534
STOPSIGNAL SIGINT
ENTRYPOINT ["/usr/local/bin/aice"]
