package main

import (
	"context"
	"github.com/coder/websocket"
	"github.com/sirupsen/logrus"
	"sync"
	"time"
)

type Websocket struct {
	URL      string
	ctx      context.Context
	failChan chan error
	conn     *websocket.Conn
	rw       sync.RWMutex
}

func NewWebSocket(ctx context.Context, url string) *Websocket {
	conn := restyWS(url)
	ws := &Websocket{
		URL:      url,
		ctx:      ctx,
		conn:     conn,
		failChan: make(chan error),
	}
	go ws.monitorReConnect()
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

func (ws *Websocket) monitorReConnect() {
	for err := range ws.failChan {
		logrus.Warnf("websocket  error :%v", err.Error())
		ws.rw.Lock()
		conn := restyWS(ws.URL)
		logrus.Infof("websocket  success")
		ws.conn = conn
		ws.rw.Unlock()
	}
	return
}

func (ws *Websocket) Write(ctx context.Context, typ websocket.MessageType, p []byte) {
	for {
		ws.rw.Lock()
		conn := ws.conn
		ws.rw.Unlock()
		err := conn.Write(ctx, typ, p)
		if err != nil {
			ws.failChan <- err
		} else {
			break
		}
	}
}

func (ws *Websocket) Read(ctx context.Context) (websocket.MessageType, []byte) {
	for {
		ws.rw.Lock()
		conn := ws.conn
		ws.rw.Unlock()
		typ, p, err := conn.Read(ctx)
		if err != nil {
			ws.failChan <- err

		} else {
			return typ, p
		}
	}
}
