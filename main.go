package main

import (
	"bytes"
	"context"
	"fmt"
	"github.com/sirupsen/logrus"
	"io"
	"os"
	"os/exec"
	"runtime"
	"strings"
)

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

func uploadRun(sourceCode string, outputFile string) string {
	err := os.WriteFile(outputFile, []byte(sourceCode), 0655)
	if err != nil {
		return "write file error"
	}
	return "upload success"
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

func TTyShell(shellType string, cat *NapCat, msgChan <-chan *NapCatResponse) {
	delim := "__CMD_DONE__"
	stdoutReader, stdoutWriter := io.Pipe()
	stderrReader, stderrWriter := io.Pipe()
	outReader := NewDelimitedReader(stdoutReader, delim)
	errReader := NewDelimitedReader(stderrReader, delim)

	execShell := exec.Command(shellType)
	stdin, err := execShell.StdinPipe()
	execShell.Stdout = stdoutWriter
	execShell.Stderr = stderrWriter
	if err != nil {
		return
	}
	startedChan := make(chan error)

	go func() {
		err := execShell.Start()
		startedChan <- err
		if err = execShell.Wait(); err != nil {
			logrus.Infof("%s wait: %s", execShell, err)
		}
	}()

	if err = <-startedChan; err != nil {
		return
	}

	// load sh profile
	io.WriteString(stdin, "source /root/.shrc")

	for resp := range msgChan {
		fullCmd := fmt.Sprintf("%s; echo %s; echo %s 1>&2\n", resp.RawMessage, delim, delim)
		logrus.Debugf("full cmd '%s' ", fullCmd[:len(fullCmd)-1])
		io.WriteString(stdin, fullCmd)
		output, _ := GetStdoutOrStderr(outReader)
		errOutput, _ := GetStdoutOrStderr(errReader)
		sendMsg := judgeOutput(nil, output, errOutput)
		cat.send(resp.GroupID, resp.UserID, sendMsg)
	}
	if err = execShell.Process.Kill(); err != nil {
		logrus.Warnf("kill shell %s,pid %d error", shellType, execShell.Process.Pid)
	}
}

func main() {

	cat := NewNapCat(context.Background(), GlobalCfg.URL)
	cmdDisPatcher := NewBuildDisPatcher()
	messageChan := make(map[Session]chan *NapCatResponse)

	for {
		var body NapCatResponse
		if err := cat.recv(&body); err != nil {
			continue
		}

		if len(body.Message) == 0 {
			continue
		}

		rawMessage := body.Message[0].Data.Text

		tagID := Session{
			GroupID: body.GroupID,
			UserID:  body.UserID,
		}

		if tagChan, ok := messageChan[tagID]; ok {
			if body.RawMessage == "exit" {
				msg := fmt.Sprintf("(%s) goodbye.", body.Sender.Nickname)
				cat.send(body.GroupID, body.UserID, msg)
				close(tagChan)
				delete(messageChan, tagID)
				continue
			}
			tagChan <- &body
		}

		if !strings.HasPrefix(rawMessage, GlobalCfg.Prefix) {
			continue
		}

		rawMessage = rawMessage[len(GlobalCfg.Prefix):]
		rawMessage = strings.TrimSpace(rawMessage)

		msg, err := parseMessage(rawMessage)

		if err != nil {
			cat.send(body.GroupID, body.UserID, "parse judge command error")
			logrus.Warnf(err.Error())
			continue
		}

		go func() {
			data := cmdDisPatcher.Run(msg)

			switch msg.Type {
			case "add":
				//todo

			case "bash", "sh", "zsh":
				manyMsg := fmt.Sprintf("(%s): %s", body.Sender.Nickname, data)
				cat.send(body.GroupID, body.UserID, manyMsg)
				logrus.Infof("%s shell start,userID %d,groupID %d,name %s", msg.Type, body.UserID, body.GroupID, body.Sender.Nickname)
				tagChan := make(chan *NapCatResponse, 100)
				messageChan[tagID] = tagChan
				shellType := msg.Type
				go TTyShell(shellType, cat, messageChan[tagID])
			default:
				cat.send(body.GroupID, body.UserID, data)

			}
		}()

		cat.send(body.GroupID, body.UserID, "building...")
	}

}
