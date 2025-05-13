package main

import (
	"context"
	"fmt"
	"strings"
	"sync"
)

type Session struct {
	GroupID int
	UserID  int
}

type MsgQueue struct {
	q     sync.Map
	queue map[Session]chan *NapCatResponse
}

func (m *MsgQueue) Store(k Session, v chan *NapCatResponse) {
	m.q.Store(k, v)
}

func (m *MsgQueue) Load(k Session) (chan *NapCatResponse, bool) {
	r, ok := m.q.Load(k)
	if !ok {
		return nil, ok
	}
	return r.(chan *NapCatResponse), ok
}

func (m *MsgQueue) Delete(k Session) {
	m.q.Delete(k)
}

func main() {

	cat := NewNapCat(context.Background(), GlobalCfg.URL)
	msgQueue := &MsgQueue{
		queue: make(map[Session]chan *NapCatResponse),
	}

	cmdDisPatcher := NewCommandDisPatcher(cat, msgQueue)
	cmdDisPatcher.Register("bash", ShellCmd)
	cmdDisPatcher.Register("sh", ShellCmd)
	cmdDisPatcher.Register("zsh", ShellCmd)
	cmdDisPatcher.Register("file", FileCmd)

	for {
		var body NapCatResponse
		if err := cat.recv(&body); err != nil {
			continue
		}

		if len(body.Message) == 0 {
			continue
		}

		rawMessage := body.Message[0].Data.Text

		sessionID := Session{
			GroupID: body.GroupID,
			UserID:  body.UserID,
		}

		if msgChan, ok := msgQueue.Load(sessionID); ok {
			if body.RawMessage == "exit" {
				msg := fmt.Sprintf("(%s) goodbye.", body.Sender.Nickname)
				cat.send(body.GroupID, body.UserID, msg)
				close(msgChan)
				msgQueue.Delete(sessionID)
				continue
			}
			msgChan <- &body
		}

		if !strings.HasPrefix(rawMessage, GlobalCfg.Prefix) {
			continue
		}

		go func() {
			output := cmdDisPatcher.Run(&body)
			cat.send(body.GroupID, body.UserID, output)
		}()

	}

}
