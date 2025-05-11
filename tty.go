package main

import (
	"fmt"
	"github.com/sirupsen/logrus"
	"io"
	"os/exec"
	"strings"
)

func CombineOutput(stdout *DelimitedReader, stderr *DelimitedReader, delim []byte) string {
	outputStdout, err1 := stdout.ReadString(delim)
	outputStderr, err2 := stderr.ReadString(delim)

	if err1 == nil && err2 == nil {
		output := fmt.Sprintf("%s\n%s", outputStdout, outputStderr)
		output = strings.TrimSpace(output)
		return output
	}
	return "get cmd stdout/stderr failed"
}

func TTyShell(shellType string, cat *NapCat, msgChan <-chan *NapCatResponse) {
	delim := []byte("__CMD_DONE__")
	stdoutReader, stdoutWriter := io.Pipe()
	stderrReader, stderrWriter := io.Pipe()
	outReader := NewDelimitedReader(stdoutReader)
	errReader := NewDelimitedReader(stderrReader)

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
	io.WriteString(stdin, "source /root/.shrc\n")

	for resp := range msgChan {
		fullCmd := fmt.Sprintf("%s; echo %s; echo %s 1>&2\n", resp.RawMessage, delim, delim)
		logrus.Debugf("full cmd '%s' ", fullCmd[:len(fullCmd)-1])
		io.WriteString(stdin, fullCmd)

		msg := CombineOutput(outReader, errReader, delim)
		cat.send(resp.GroupID, resp.UserID, msg)
	}
	if err = execShell.Process.Kill(); err != nil {
		logrus.Warnf("kill shell %s,pid %d error", shellType, execShell.Process.Pid)
	}
}
