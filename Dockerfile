# Build stage
FROM rust:1.88-alpine AS builder
LABEL authors="fordkuppp"
WORKDIR /app

RUN apk update && apk add --no-cache build-base
COPY . .
RUN cargo build --release

# Runner stage
FROM alpine:3.22.1 AS runner
WORKDIR /usr/local/steam-metrics

COPY --from=builder /app/target/release/steam-metrics ./
COPY config/default.toml ./config/default.toml

CMD ["./steam-metrics"]