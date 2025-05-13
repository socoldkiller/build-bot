package main

import (
	"context"
	"fmt"
	"github.com/google/shlex"
	"os"
	"strings"
	"sync"
)

type BuildMessage struct {
	SubCommand string
	Args       []string
	SourceCode string
}

type MessageContext struct {
	buildMsg          *BuildMessage
	cat               *NapCat
	id                *Session
	rawMsg            *NapCatResponse
	msgContext        chan *NapCatResponse
	msgSessionContext *MsgQueue
	commands          map[string]string
}

type CommandDispatcher struct {
	commands    map[string]string
	commandRuns map[string]func(msgContext *MessageContext) (string, error)
	rw          sync.RWMutex
	msgChan     *MsgQueue
	cat         *NapCat
}

func parseMessage(raw string) (*BuildMessage, error) {
	raws := strings.SplitAfterN(raw, "\n", 2)

	if len(raws) == 0 {
		return nil, fmt.Errorf("%s", "parse error,raws must >=1")
	}
	tokens, _ := shlex.Split(raws[0])
	if len(tokens) < 2 {
		return nil, fmt.Errorf("parse error")
	}
	Type := tokens[1]
	Args := tokens[2:]
	var SourceCode string
	if len(raws) == 2 {
		SourceCode = raws[1]
	}

	return &BuildMessage{
		SubCommand: Type,
		Args:       Args,
		SourceCode: SourceCode,
	}, nil
}

func NewCommandDisPatcher(cat *NapCat, msgChan *MsgQueue) *CommandDispatcher {
	return &CommandDispatcher{
		commands:    make(map[string]string),
		commandRuns: make(map[string]func(msgCtx *MessageContext) (string, error)),
		msgChan:     msgChan,
		cat:         cat,
	}
}

func (b *CommandDispatcher) Run(rawMsg *NapCatResponse) string {
	rawMessage := rawMsg.Message[0].Data.Text
	rawMessage = rawMessage[len(GlobalCfg.Prefix):]
	rawMessage = strings.TrimSpace(rawMessage)

	msg, err := parseMessage(rawMessage)
	if err != nil {
		return "parse message error!"
	}

	b.rw.Lock()
	defer b.rw.Unlock()
	id := Session{
		GroupID: rawMsg.GroupID,
		UserID:  rawMsg.UserID,
	}
	msgContext, _ := GetMsgQueue(id, b.msgChan)
	msgCtx := &MessageContext{
		msgContext:        msgContext,
		cat:               b.cat,
		commands:          b.commands,
		buildMsg:          msg,
		id:                &id,
		rawMsg:            rawMsg,
		msgSessionContext: b.msgChan,
	}

	if run, ok := b.commandRuns[msg.SubCommand]; ok {
		data, err := run(msgCtx)
		if err != nil {
			return err.Error()
		}
		return data
	}

	return fmt.Sprintf("not found subcommand %s", msg.SubCommand)
}

func (b *CommandDispatcher) Register(subCommand string, f func(msgCtx *MessageContext) (string, error)) {
	b.rw.Lock()
	defer b.rw.Unlock()
	b.commandRuns[subCommand] = f
}

func ShellCmd(msgCtx *MessageContext) (string, error) {
	cat := msgCtx.cat
	name := msgCtx.rawMsg.Sender.Nickname
	title := fmt.Sprintf("(%s):qq %s terminal start", name, msgCtx.buildMsg.SubCommand)

	removeSession := func(sessionMsgContext *MsgQueue, stopChan <-chan error) {
		<-stopChan
		DeleteMsgQueue(*msgCtx.id, msgCtx.msgSessionContext)
	}

	shell, err := NewShell(msgCtx.buildMsg.SubCommand)
	if err != nil {
		return "", fmt.Errorf("create shell %s failed,err: %v", msgCtx.buildMsg.SubCommand, err)
	}
	msgCtx.msgContext = make(chan *NapCatResponse, 100)
	SetMsgQueue(*msgCtx.id, msgCtx.msgContext, msgCtx.msgSessionContext)

	stopChan := make(chan error)
	go TTyShell(context.Background(), shell, cat, msgCtx.msgContext, stopChan)
	go removeSession(msgCtx.msgSessionContext, stopChan)
	return title, nil
}

func FileCmd(msgCtx *MessageContext) (string, error) {
	if len(msgCtx.buildMsg.Args) == 0 {
		return "", nil
	}
	file := msgCtx.buildMsg.Args[0]
	err := os.WriteFile(file, []byte(msgCtx.buildMsg.SourceCode), 0655)
	if err != nil {
		return "", err
	}
	return "write file success", nil
}
