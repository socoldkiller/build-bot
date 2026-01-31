
FROM rustlang/rust:nightly-alpine AS builder

RUN apk add --no-cache musl-dev  upx

WORKDIR /app

COPY . .

RUN cargo build --release && \
    upx --best --lzma /app/target/release/build-bot


FROM alpine:3.22

WORKDIR /app

COPY --from=builder /app/target/release/build-bot /app/build-bot

CMD ["/app/build-bot","-f","config.toml"]
