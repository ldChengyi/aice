FROM rust:1.99.1-bookworm AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --locked --release --bin aice

FROM debian:bookworm-slim
COPY --from=build /app/target/release/aice /usr/local/bin/aice
ENV AICE_BIND_ADDR=0.0.0.0:3210
EXPOSE 3210
USER 65534:65534
STOPSIGNAL SIGINT
ENTRYPOINT ["/usr/local/bin/aice"]
