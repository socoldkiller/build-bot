package main

import (
	"context"
	"fmt"
	"github.com/sirupsen/logrus"
	"strings"
	"sync"
)

type Session struct {
	GroupID int
	UserID  int
}

func BuildBotLoop(cat *NapCat, msgQueue *MsgQueue, dispatcher *CommandDispatcher) error {
	var err error
	for {
		var body NapCatResponse
		if err = cat.recv(&body); err != nil {
			break
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
				if err = cat.send(body.GroupID, body.UserID, msg); err != nil {
					break
				}
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
			output := dispatcher.Run(&body)
			if err := cat.send(body.GroupID, body.UserID, output); err != nil {
				//TODO
			}
		}()

	}

	return err
}

func main() {
	catChan := make(chan *NapCat)
	failChan := make(chan error, 1)
	var wg sync.WaitGroup
	wg.Add(1)
	go func(ctx context.Context, catChan chan *NapCat, failChan chan error) {
		defer wg.Done()
		msgQueue := &MsgQueue{}
		for {
			select {
			case <-ctx.Done():
				return

			case cat, ok := <-catChan:
				if !ok {
					return
				}
				cmdDisPatcher := NewCommandDisPatcher(cat, msgQueue)
				cmdDisPatcher.Register("bash", ShellCmd)
				cmdDisPatcher.Register("sh", ShellCmd)
				cmdDisPatcher.Register("zsh", ShellCmd)
				cmdDisPatcher.Register("file", FileCmd)
				err := BuildBotLoop(cat, msgQueue, cmdDisPatcher)
				logrus.Warnf("connect failed: %s", err)
				failChan <- err
			}
		}
	}(context.Background(), catChan, failChan)

	wg.Add(1)
	go func(ctx context.Context, catChan chan *NapCat, failChan chan error) {
		defer wg.Done()
		for {
			select {
			case <-ctx.Done():
				return
			case <-failChan:
				conn := NewWebSocket(context.Background(), GlobalCfg.URL)
				cat := NewNapCat(conn)
				catChan <- cat
			}

		}

	}(context.Background(), catChan, failChan)
	failChan <- nil
	wg.Wait()

}
