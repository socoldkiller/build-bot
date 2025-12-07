
FROM rustlang/rust:nightly-alpine AS builder

RUN apk add --no-cache musl-dev openssl-dev

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY build.rs ./

RUN cargo build --release

FROM alpine:3.22

RUN apk add --no-cache libgcc openssl


WORKDIR /app

COPY --from=builder /app/target/release/build-bot /app/build-bot


CMD ["/app/build-bot","-f","config.toml"]
