package main

import (
	"fmt"
	"strings"
)

type BuildMessage struct {
	Type       string
	Args       []string
	SourceCode string
}

func parseMessage(raw string) (*BuildMessage, error) {
	raws := strings.SplitAfterN(raw, "\n", 2)

	if len(raws) == 0 {
		return nil, fmt.Errorf("%s", "parse error,raws must >=1")
	}

	tokens := strings.Fields(raws[0])
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
		Type:       Type,
		SourceCode: SourceCode,
		Args:       Args,
	}, nil
}

type BuildDisPatcher struct {
	buildMsg BuildMessage
}

func (b *BuildDisPatcher) NewBuildDisPatcher(cmd string) {
}

func (b *BuildDisPatcher) Run(msg *BuildMessage) string {
	var output string
	name, err := memfdCreate("output")
	if err != nil {
		return output
	}

	switch msg.Type {
	case "bash", "sh", "zsh":
		//todo
		output = fmt.Sprintf("qq %s terminal start", msg.Type)

	case "c++", "cpp":
		output = cppCodeRun(msg.SourceCode, name)

	case "py", "python3", "python":
		output = pyCodeRun(msg.SourceCode, name)

	case "go":
		output = goCodeRun(msg.SourceCode, name)

	case "rust":

	}
	return output
}
