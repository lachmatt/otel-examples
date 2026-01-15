FROM rust:1.91-bookworm AS builder
WORKDIR app
COPY . .
RUN cargo build --release --bins

FROM debian:bookworm AS client
RUN apt-get update && apt-get install -y ca-certificates
WORKDIR app
COPY --from=builder /app/target/release/client .
ENTRYPOINT ["./client"]

FROM debian:bookworm AS server
COPY --from=builder /app/target/release/server .
ENTRYPOINT ["./server"]