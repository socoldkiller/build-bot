package main

import (
	"bytes"
	"context"
	"fmt"
	"github.com/coder/websocket"
	"github.com/sirupsen/logrus"
	"log"
	"os"
	"runtime"
	"strings"
	"time"
)

type BuildMessage struct {
	Type       string
	Args       []string
	SourceCode string
}

func parseMessage(raw string) (*BuildMessage, error) {
	line := strings.SplitAfterN(raw, "\n", 2)

	if len(line) == 0 {
		return nil, fmt.Errorf("%s", "parse error,line must >=1")
	}

	tokens := strings.Fields(line[0])

	if len(tokens) < 2 {
		return nil, fmt.Errorf("parse error")
	}

	Type := tokens[2]
	Args := tokens[3:]

	SourceCode := line[1]
	return &BuildMessage{
		Type:       Type,
		SourceCode: SourceCode,
		Args:       Args,
	}, nil
}

func executeFileName(name string) string {
	switch runtime.GOOS {
	case "linux":
		return name
	case "darwin":
		return fmt.Sprintf("./%s", name)
	case "windows":
	default:
	}
	return ""
}

func cppCodeRun(sourceCode string, outputFile string) string {
	var (
		stdout bytes.Buffer
		stderr bytes.Buffer
		stdin  = strings.NewReader(sourceCode)
		err    error
	)

	r := CmdRunner{
		cmd:  "g++",
		args: []string{"-std=c++17", "-x", "c++", "-o", outputFile, "-"},
	}
	err = r.Run(stdin, &stdout, &stderr)
	buildMsg := judgeOutput(err, stdout.String(), stderr.String())

	stdout.Reset()
	stderr.Reset()

	r = CmdRunner{
		cmd: executeFileName(outputFile),
	}

	err = r.Run(nil, &stdout, &stderr)

	output := judgeOutput(err, stdout.String(), stderr.String())
	if strings.Contains(output, "no such file or directory ") || strings.Contains(output, "exec format error") {
		return buildMsg
	}
	return output

}

func goCodeRun(sourceCode string, outputFile string) string {
	var (
		stdout bytes.Buffer
		stderr bytes.Buffer
		err    error
	)

	sourceFile := "main.go"
	defer os.Remove(sourceFile)

	err = os.WriteFile(sourceFile, []byte(sourceCode), 0655)
	if err != nil {
		return ""
	}

	r := CmdRunner{
		cmd:  "go",
		args: []string{"build", "-o", outputFile, sourceFile},
	}

	err = r.Run(nil, &stdout, &stderr)
	buildMsg := judgeOutput(err, stdout.String(), stderr.String())

	stdout.Reset()
	stderr.Reset()

	r = CmdRunner{
		cmd: executeFileName(outputFile),
	}

	err = r.Run(nil, &stdout, &stderr)

	output := judgeOutput(err, stdout.String(), stderr.String())
	if strings.Contains(output, "no such file or directory ") || strings.Contains(output, "exec format error") {
		return buildMsg
	}
	return output

}

func pyCodeRun(sourceCode string, outputFile string) string {
	var (
		stdout bytes.Buffer
		stderr bytes.Buffer
		stdin  = strings.NewReader(sourceCode)
		err    error
	)
	r := CmdRunner{
		cmd: "python3",
	}

	err = r.Run(stdin, &stdout, &stderr)
	output := judgeOutput(err, stdout.String(), stderr.String())
	return output
}

func outputMessage(raw string, outputFileName string) string {
	msg, err := parseMessage(raw)
	if err != nil {
		return err.Error()
	}
	var output string
	switch msg.Type {

	case "c++", "cpp":
		output = cppCodeRun(msg.SourceCode, outputFileName)

	case "py", "python3", "python":
		output = pyCodeRun(msg.SourceCode, outputFileName)

	case "go":
		output = goCodeRun(msg.SourceCode, outputFileName)

	case "rust":

	}

	return output

}

func ErrOutput(prefix string, err error) {
	if err != nil {
		logrus.Warnf("%s :%v", prefix, err)
	}
}

func HelpOutput() {

}

func main() {
	ctx := context.Background()

	conn, _, err := websocket.Dial(ctx, "ws://127.0.0.1:3001/?access_token=napcat", nil)
	if err != nil {
		log.Fatal("dial error:", err)
	}
	defer conn.Close(websocket.StatusInternalError, "closing")

	cat := &NapCat{conn: conn}
	for {

		var body NapCatResponse
		if err := cat.recv(&body); err != nil {
			continue
		}

		if (body.UserID != 0 || body.GroupID != 0) && strings.HasPrefix(body.RawMessage, "大鱼鱼") {
			var tokens []string
			if tokens = strings.Fields(body.RawMessage); len(tokens) < 2 || tokens[1] != "judge" {
				continue
			}

			outputChan := make(chan error)
			go func() {
				name, err := memfdCreate("output")
				if err != nil {
					return
				}
				output := outputMessage(body.RawMessage, name)
				err = cat.send(body.GroupID, body.UserID, output)
				outputChan <- err
				ErrOutput("send error", err)
			}()

			select {
			case <-outputChan:

			case <-time.After(100 * time.Millisecond):
				err := cat.send(body.GroupID, body.UserID, "building...")
				ErrOutput("send error", err)
				<-outputChan

			}
		}

	}

}
