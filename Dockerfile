
FROM golang:1.24-alpine as builder

COPY . /app

WORKDIR /app

RUN go build -o build-bot


FROM alpine

COPY --from=builder /app/build-bot /app/build-bot

RUN apk add python3 gcc go g++ musl-dev


CMD /app/build-bot