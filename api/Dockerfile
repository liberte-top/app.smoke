FROM rust:1.91-bookworm AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo fetch
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12
WORKDIR /app
COPY --from=builder /app/target/release/app-smoke-api /app/app-smoke-api

USER nonroot:nonroot
EXPOSE 4310
ENTRYPOINT ["/app/app-smoke-api"]
