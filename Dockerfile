
FROM golang:1.24.2-alpine as builder

COPY . /app

WORKDIR /app

RUN apk update && \
    apk add upx

RUN CGO_ENABLED=0 go build -ldflags "-s -w" -gcflags="all=-N -l" -o build-bot

RUN upx -9 build-bot


FROM alpine

RUN apk add --no-cache tzdata \
    && cp /usr/share/zoneinfo/Asia/Shanghai /etc/localtime \
    && echo "Asia/Shanghai" > /etc/timezone \
    && apk del tzdata

COPY --from=builder /app/build-bot /app/build-bot


WORKDIR /app
COPY entrypoint.sh entrypoint.sh

ENTRYPOINT ["./entrypoint.sh"]