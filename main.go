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
	rw    sync.Mutex
	queue map[Session]chan *NapCatResponse
}

func GetMsgQueue(session Session, q *MsgQueue) (chan *NapCatResponse, bool) {
	q.rw.Lock()
	defer q.rw.Unlock()
	msgChan, ok := q.queue[session]
	return msgChan, ok
}

func SetMsgQueue(session Session, msgChan chan *NapCatResponse, q *MsgQueue) {
	q.rw.Lock()
	defer q.rw.Unlock()
	q.queue[session] = msgChan
}

func DeleteMsgQueue(session Session, q *MsgQueue) {
	q.rw.Lock()
	defer q.rw.Unlock()
	delete(q.queue, session)
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

		if msgChan, ok := GetMsgQueue(sessionID, msgQueue); ok {
			if body.RawMessage == "exit" {
				msg := fmt.Sprintf("(%s) goodbye.", body.Sender.Nickname)
				cat.send(body.GroupID, body.UserID, msg)
				close(msgChan)
				DeleteMsgQueue(sessionID, msgQueue)
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
