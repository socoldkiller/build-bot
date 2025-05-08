
FROM golang:1.24.2-alpine as builder

COPY . /app

WORKDIR /app

RUN CGO_ENABLED=0 go build -ldflags "-s -w" -gcflags="all=-N -l" -o build-bot


FROM alpine

COPY --from=builder /app/build-bot /app/build-bot

WORKDIR /app
COPY entrypoint.sh entrypoint.sh

ENTRYPOINT ["./entrypoint.sh"]