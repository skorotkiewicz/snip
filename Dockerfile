# Build stage
FROM rust:alpine AS builder

RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

RUN cargo build --release

# Runtime stage
FROM alpine:latest

RUN apk add --no-cache ca-certificates

WORKDIR /app

# Copy binaries and static files
COPY --from=builder /app/target/release/snipped /app/snipped
COPY --from=builder /app/target/release/snip /app/snip
COPY --from=builder /app/migrations ./migrations
COPY --from=builder /app/src/index.html ./src/index.html

# Create data directory for SQLite
RUN mkdir -p /data

ENV DATABASE_URL=sqlite:/data/snip.db
ENV RUST_LOG=info

EXPOSE 3000

VOLUME ["/data"]

CMD ["./snipped"]
