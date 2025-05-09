package main

import (
	"fmt"
	"github.com/sirupsen/logrus"
	"io"
	"os/exec"
)

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
	io.WriteString(stdin, "source /root/.shrc\n")

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
