package main

import (
	"bytes"
	"context"
	"fmt"
	"github.com/coder/websocket"
	"github.com/sirupsen/logrus"
	"log"
	"os/exec"
	"strings"
	"time"
)

func execCommand(stdin string, cmd string, args ...string) string {
	var (
		stdout bytes.Buffer
		stderr bytes.Buffer
	)

	runner := exec.Command(cmd, args...)
	runner.Stdin = strings.NewReader(stdin)
	runner.Stdout = &stdout
	runner.Stderr = &stderr
	err := runner.Run()

	buildErrMsg := stderr.String()
	runner = exec.Command("./output")
	runner.Stdout = &stdout
	runner.Stderr = &stderr

	err = runner.Run()
	if err != nil {
		if buildErrMsg != "" {
			return buildErrMsg
		}
		return err.Error()
	}

	if stderr.String() != "" {
		return stderr.String()
	}

	return stdout.String()

}

func PyExecCommand(stdin string, cmd string, args ...string) string {
	var (
		stdout bytes.Buffer
		stderr bytes.Buffer
	)

	runner := exec.Command(cmd, args...)
	runner.Stdin = strings.NewReader(stdin)
	runner.Stdout = &stdout
	runner.Stderr = &stderr
	err := runner.Run()

	if err == nil {
		return stdout.String()
	}

	return stderr.String()
}

func GoExecCommand(stdin string, cmd string, args ...string) string {
	var (
		stdout bytes.Buffer
		stderr bytes.Buffer
	)

	runner := exec.Command(cmd, args...)
	runner.Stdin = strings.NewReader(stdin)
	runner.Stdout = &stdout
	runner.Stderr = &stderr
	err := runner.Run()

	if err == nil {
		return stdout.String()
	}

	return stderr.String()
}

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

	Type := tokens[1]
	SourceCode := line[1]
	Args := tokens[2:]

	return &BuildMessage{
		Type:       Type,
		SourceCode: SourceCode,
		Args:       Args,
	}, nil

}

func outputMessage(raw string) string {
	msg, err := parseMessage(raw)
	if err != nil {
		return err.Error()
	}
	var (
		output string
		stdout bytes.Buffer
		stderr bytes.Buffer
		stdin  = strings.NewReader(msg.SourceCode)
	)
	r := CmdRunner{
		cmd:  msg.Args[0],
		args: msg.Args[1:],
	}

	err = r.Run(stdin, &stdout, &stderr)
	output = judgeOutput(err, stdout.String(), stderr.String())
	return output
}

func ErrOutput(prefix string, err error) {
	if err != nil {
		logrus.Warnf("%s :%v", prefix, err)
	}
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
			outputChan := make(chan error)
			go func() {
				output := outputMessage(body.RawMessage)
				err = cat.send(body.GroupID, body.UserID, output)
				outputChan <- err
				ErrOutput("send error", err)
			}()

			select {
			case <-outputChan:

			case <-time.After(100 * time.Millisecond):
				err := cat.send(body.GroupID, body.UserID, "building...")
				ErrOutput("send error", err)

			}
		}

	}

}
