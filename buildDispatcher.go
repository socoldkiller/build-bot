package main

import (
	"fmt"
	"github.com/samber/lo"
	"os"
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

	commands map[string]string
}

func NewBuildDisPatcher() *BuildDisPatcher {
	return &BuildDisPatcher{
		commands: make(map[string]string),
	}
}

func parseAliasCommands(msg *BuildMessage) map[string]string {
	commands := make(map[string]string)
	for _, arg := range msg.Args {
		splitter := strings.SplitN(arg, "=", 2)

		if len(splitter) != 2 {
			continue
		}

		varName := splitter[0]
		varValue := splitter[1]
		commands[varName] = varValue
	}
	return commands
}

func exportCommand(fileName string, commands map[string]string) error {
	f, err := os.OpenFile(fileName, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0644)
	if err != nil {
		return err
	}
	defer f.Close()
	for k, v := range commands {
		cmd := fmt.Sprintf("alias %s=%s\n", k, v)
		f.WriteString(cmd)
	}

	return nil
}

func aliasCommandRun(patcher *BuildDisPatcher, msg *BuildMessage) string {
	if len(msg.Args) == 0 {
		return ""
	}
	var output string
	aliasCommands := parseAliasCommands(msg)
	for k, v := range aliasCommands {
		patcher.commands[k] = v
	}
	err := exportCommand("/root/.shrc", aliasCommands)
	if err != nil {
		return err.Error()
	}
	output = fmt.Sprintf("✅ Command %v added successfully!", lo.Keys(aliasCommands))
	return output
}

func (b *BuildDisPatcher) Run(msg *BuildMessage) string {
	var output string

	name, err := memfdCreate("output")
	if err != nil {
		return output
	}

	switch msg.Type {
	case "alias":
		output = aliasCommandRun(b, msg)

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

	case "upload":
		output = uploadRun(msg.SourceCode, msg.Args[0])

	case "list":
		output = b.showAllCommands()

	}
	return output
}

func (b *BuildDisPatcher) showAllCommands() string {
	output := ""
	for cmd, _ := range b.commands {
		output += fmt.Sprintf("%s\n", cmd)
	}
	return output

}
