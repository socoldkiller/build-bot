package main

import (
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
	buildMsg   *BuildMessage
	cat        *NapCat
	id         *Session
	rawMsg     *NapCatResponse
	msgContext chan *NapCatResponse
	commands   map[string]string
}

type CommandDispatcher struct {
	commands    map[string]string
	commandRuns map[string]func(msgContext *MessageContext) string
	rw          sync.RWMutex

	msgChan map[Session]chan *NapCatResponse
	cat     *NapCat
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

func NewCommandDisPatcher(cat *NapCat, msgChan map[Session]chan *NapCatResponse) *CommandDispatcher {
	return &CommandDispatcher{
		commands:    make(map[string]string),
		commandRuns: make(map[string]func(msgCtx *MessageContext) string),
		msgChan:     msgChan,
		cat:         cat,
	}
}

func (b *CommandDispatcher) Run(rawMsg *NapCatResponse) string {
	rawMessage := rawMsg.RawMessage
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

	msgCtx := &MessageContext{
		msgContext: b.msgChan[id],
		cat:        b.cat,
		commands:   b.commands,
		buildMsg:   msg,
		id:         &id,
		rawMsg:     rawMsg,
	}

	if run, ok := b.commandRuns[msg.SubCommand]; ok {
		if _, ok1 := b.msgChan[id]; !ok1 {
			if msg.SubCommand == "bash" || msg.SubCommand == "sh" || msg.SubCommand == "zsh" {
				b.msgChan[id] = make(chan *NapCatResponse, 100)
				msgCtx.msgContext = b.msgChan[id]
			}
		}
		data := run(msgCtx)
		return data
	}

	return fmt.Sprintf("not found subcommand %s", msg.SubCommand)
}

func (b *CommandDispatcher) Register(subCommand string, f func(msgCtx *MessageContext) string) {
	b.rw.Lock()
	defer b.rw.Unlock()
	b.commandRuns[subCommand] = f
}

func ShellCmd(msgCtx *MessageContext) string {
	cat := msgCtx.cat
	name := msgCtx.rawMsg.Sender.Nickname
	title := fmt.Sprintf("(%s):qq %s terminal start", name, msgCtx.buildMsg.SubCommand)
	go TTyShell(msgCtx.buildMsg.SubCommand, cat, msgCtx.msgContext)
	return title
}

func FileCmd(msgCtx *MessageContext) string {
	if len(msgCtx.buildMsg.Args) == 0 {
		return ""
	}
	file := msgCtx.buildMsg.Args[0]
	err := os.WriteFile(file, []byte(msgCtx.buildMsg.SourceCode), 0655)
	if err != nil {
		return err.Error()
	}
	return "write file success"
}
