package main

import (
	"context"
	"github.com/coder/websocket"
	"github.com/sirupsen/logrus"
	"time"
)

type Websocket struct {
	URL  string
	ctx  context.Context
	conn *websocket.Conn
}

func NewWebSocket(ctx context.Context, url string) *Websocket {
	ws := &Websocket{
		URL:  url,
		ctx:  ctx,
		conn: restyWS(url),
	}
	return ws
}

func restyWS(url string) *websocket.Conn {
	delay := 1 * time.Second
	restartDelay := func(duration *time.Duration) {
		if *duration > 1*time.Minute {
			*duration = 1 * time.Second
		}
	}
	for {
		ctx := context.Background()
		conn, _, err := websocket.Dial(ctx, url, nil)
		if err == nil {
			logrus.Infof("websocket connect success")
			// close limiter
			conn.SetReadLimit(-1)
			return conn
		}
		logrus.Warnf("websocket connect failed :%v", err.Error())
		time.Sleep(delay)
		delay = delay * 2
		restartDelay(&delay)
	}
}

func (ws *Websocket) Write(ctx context.Context, typ websocket.MessageType, p []byte) {
	if err := ws.conn.Write(ctx, typ, p); err != nil {
		logrus.Warnf("websocket conn can't write msg:(%s)", string(p))
	}
}

func (ws *Websocket) Read(ctx context.Context) (websocket.MessageType, []byte) {
	var (
		err  error
		typ  websocket.MessageType
		body []byte
	)

	if typ, body, err = ws.conn.Read(ctx); err != nil {
		logrus.Warnf("websocket conn can't read message")
	}
	return typ, body
}
