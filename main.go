package main

import (
	"context"
	"fmt"
	"io"
	"strings"
)

type Session struct {
	GroupID int
	UserID  int
}

func GetStdoutOrStderr(reader io.Reader) (string, error) {
	output, err := io.ReadAll(reader)
	if err != nil {
		return "", err
	}
	str := string(output)
	return strings.TrimSpace(str), nil
}

func main() {

	cat := NewNapCat(context.Background(), GlobalCfg.URL)
	msgQueue := make(map[Session]chan *NapCatResponse)
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

		if msgChan, ok := msgQueue[sessionID]; ok {
			if body.RawMessage == "exit" {
				msg := fmt.Sprintf("(%s) goodbye.", body.Sender.Nickname)
				cat.send(body.GroupID, body.UserID, msg)
				close(msgChan)
				delete(msgQueue, sessionID)
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
